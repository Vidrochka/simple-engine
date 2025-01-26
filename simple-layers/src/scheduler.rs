use std::collections::HashMap;

use tokio::runtime::Handle;
use xdi::{IAsyncTaskScope, ServiceProvider, types::error::ServiceBuildResult};

use crate::{
    layer::LayerCtx,
    types::{
        id::LayerId,
        sync::{Waiter, signal_channel},
    },
};

#[derive(Debug)]
pub struct LayerScheduler {
    handler: Handle,
    sp: ServiceProvider,
    scheduled_tasks: HashMap<LayerId, Vec<Waiter>, ahash::RandomState>,
    scheduled_tasks_by_name: HashMap<String, LayerId>,
}

impl LayerScheduler {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        let handler = sp.resolve::<Handle>()?;

        Ok(Self {
            handler,
            sp,
            scheduled_tasks: Default::default(),
            scheduled_tasks_by_name: Default::default(),
        })
    }

    /// Schedule a task to be performed after another layer's tasks are completed
    pub fn schedule<'a, const DEPENDENCY_COUNT: usize>(
        &mut self,
        task: impl Future<Output = ()> + Send + Sync + 'static,
        deps: impl Into<Dependency<'a, DEPENDENCY_COUNT>>,
    ) -> Waiter {
        let lc = self.sp.resolve::<LayerCtx>().unwrap();

        self.scheduled_tasks_by_name.insert(lc.name(), lc.id());
        
        let (wk, wt) = signal_channel();
        
        let deps = deps.into();

        let deps_waiters = match deps {
            Dependency::IdList(ids) => {
                ids.iter()
                    .filter_map(|id| self.scheduled_tasks.get(id).cloned())
                    .flatten()
                    .collect::<Vec<_>>()
            }
            Dependency::SizedNameList(names) => {
                names.iter()
                    .filter_map(|name| self.scheduled_tasks_by_name.get(*name))
                    .filter_map(|id| self.scheduled_tasks.get(id).cloned())
                    .flatten()
                    .collect::<Vec<_>>()
            }
            Dependency::None => {
                Vec::new()
            }
        };

        self.handler.spawn(
            async move {
                for deps_waiter in deps_waiters {
                    deps_waiter.wait().await;
                }

                task.await;

                wk.signal().await;
            }
            .add_service_span(),
        );

        self.scheduled_tasks
            .entry(lc.id())
            .and_modify(|waters| waters.push(wt.clone()))
            .or_insert(Vec::from([wt.clone()]));

        wt
    }

    /// Wait for all scheduled tasks to complete
    pub fn wait_all_blocking(&mut self) {
        if self.scheduled_tasks.is_empty() {
            return;
        }

        self.handler.block_on(async {
            tracing::debug!("Block on waiting {count} tasks", count = self.scheduled_tasks.len());

            for (layer_id, waiters) in self.scheduled_tasks.drain() {
                for waiter in waiters {
                    waiter.wait().await;
                }
                
                tracing::debug!("{layer_id} wait completed");
            }
        });
    }
}

pub enum Dependency<'a, const N: usize> {
    IdList(&'a [LayerId]),
    SizedNameList([&'a str; N]),
    None,
}

impl<'a, const N: usize> From<[&'a str; N]> for Dependency<'a, N> {
    fn from(value: [&'a str; N]) -> Self {
        Dependency::SizedNameList(value)
    }
}

impl<'a> From<&'a [LayerId]> for Dependency<'a, 0> {
    fn from(value: &'a [LayerId]) -> Self {
        Dependency::IdList(value)
    }
}

impl<'a> From<()> for Dependency<'a, 0> {
    fn from(_: ()) -> Self {
        Dependency::None
    }
}