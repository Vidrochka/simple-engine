use async_broadcast::Sender as AsyncSender;
use std::sync::mpsc::{self, Sender as SyncSender};
use tokio::runtime::Builder;
use xdi::{ServiceProvider, builder::DiBuilder, types::error::ServiceBuildResult};

use crate::{
    ILayersSystemDependencies,
    layer::{ILayer, LayersStack},
    scheduler::LayerScheduler,
    types::id::LayerId,
};

#[derive(Debug)]
pub struct Layer {
    data: i32,
    sender: SyncSender<i32>,
}

impl Layer {
    pub fn new(data: i32, sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            data,
            sender: sp.resolve()?,
        })
    }
}

impl ILayer for Layer {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, _scheduler: &mut LayerScheduler) {
        self.sender.send(self.data).unwrap();
    }
}

#[derive(Debug)]
pub struct AsyncLayer {
    data: i32,
    sender: AsyncSender<i32>,
}

impl AsyncLayer {
    pub fn new(data: i32, sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            data,
            sender: sp.resolve()?,
        })
    }
}

impl ILayer for AsyncLayer {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, scheduler: &mut LayerScheduler) {
        let sender = self.sender.clone();
        let data = self.data;

        scheduler.schedule(
            async move {
                sender.broadcast(data).await.unwrap();
            },
            (),
        );
    }
}

#[derive(Debug)]
pub struct AsyncLayerWithDep {
    data: i32,
    dep: Vec<LayerId>,
    sender: AsyncSender<i32>,
}

impl AsyncLayerWithDep {
    pub fn new(data: i32, dep: Vec<LayerId>, sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            data,
            dep,
            sender: sp.resolve()?,
        })
    }
}

impl ILayer for AsyncLayerWithDep {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, scheduler: &mut LayerScheduler) {
        let sender = self.sender.clone();
        let data = self.data;

        scheduler.schedule(
            async move {
                sender.broadcast(data).await.unwrap();
            },
            self.dep.as_slice(),
        );
    }
}

#[test]
fn layers_stack_update_ok() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap();

    let (tx, rx) = mpsc::channel::<i32>();

    let builder = DiBuilder::new();

    let handler = runtime.handle().clone();

    builder.singletone(move |_| Ok(handler.clone()));

    builder.thread_local(move |_| Ok(tx.clone()));

    builder.register_layers_system_dependencies();

    let sp = builder.build();

    let mut stack = sp.resolve::<LayersStack>().unwrap();

    stack.push_layer("0", |sp| Ok(Layer::new(0, sp)?));

    stack.push_layer("1", |sp| Ok(Layer::new(1, sp)?));

    stack.push_layer("2", |sp| Ok(Layer::new(2, sp)?));

    stack.update();

    assert_eq!(rx.recv().unwrap(), 0);
    assert_eq!(rx.recv().unwrap(), 1);
    assert_eq!(rx.recv().unwrap(), 2);
}

#[test]
fn layers_stack_disable_update_ok() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap();

    let (tx, rx) = mpsc::channel::<i32>();

    let builder = DiBuilder::new();

    let handler = runtime.handle().clone();

    builder.singletone(move |_| Ok(handler.clone()));

    builder.thread_local(move |_| Ok(tx.clone()));

    builder.register_layers_system_dependencies();

    let sp = builder.build();

    let mut stack = sp.resolve::<LayersStack>().unwrap();

    stack.push_layer("0", |sp| Ok(Layer::new(0, sp)?));

    stack.push_layer("1", |sp| Ok(Layer::new(1, sp)?)).disable();

    stack.push_layer("2", |sp| Ok(Layer::new(2, sp)?));

    stack.update();

    assert_eq!(rx.recv().unwrap(), 0);
    assert_eq!(rx.recv().unwrap(), 2);
}

#[test]
fn layers_stack_update_with_async_task_ok() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap();

    let (tx, mut rx) = async_broadcast::broadcast::<i32>(3);

    let builder = DiBuilder::new();

    let handler = runtime.handle().clone();

    builder.singletone(move |_| Ok(handler.clone()));

    builder.thread_local(move |_| Ok(tx.clone()));

    builder.register_layers_system_dependencies();

    let sp = builder.build();

    let mut stack = sp.resolve::<LayersStack>().unwrap();

    stack.push_layer("0", |sp| Ok(AsyncLayer::new(0, sp)?));

    stack.push_layer("1", |sp| Ok(AsyncLayer::new(1, sp)?));

    stack.push_layer("2", |sp| Ok(AsyncLayer::new(2, sp)?));

    stack.update();

    runtime.block_on(async move {
        let res = vec![
            rx.recv().await.unwrap(),
            rx.recv().await.unwrap(),
            rx.recv().await.unwrap(),
        ];
        assert!(res.contains(&0));
        assert!(res.contains(&1));
        assert!(res.contains(&2));
    });
}

#[test]
fn layers_stack_update_with_async_task_with_dep_ok() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap();

    let (tx, mut rx) = async_broadcast::broadcast::<i32>(3);

    let builder = DiBuilder::new();

    let handler = runtime.handle().clone();

    builder.singletone(move |_| Ok(handler.clone()));

    builder.thread_local(move |_| Ok(tx.clone()));

    builder.register_layers_system_dependencies();

    let sp = builder.build();

    let mut stack = sp.resolve::<LayersStack>().unwrap();

    let layer_id = stack.push_layer("0", |sp| Ok(AsyncLayerWithDep::new(0, vec![], sp)?)).id();

    let layer_id = stack.push_layer(
        "1", 
        move |sp| Ok(AsyncLayerWithDep::new(1, vec![layer_id], sp)?),
    ).id();

    stack.push_layer(
        "2", 
        move |sp| Ok(AsyncLayerWithDep::new(2, vec![layer_id], sp)?),
    ).id();

    stack.update();

    runtime.block_on(async move {
        assert_eq!(rx.recv().await.unwrap(), 0);
        assert_eq!(rx.recv().await.unwrap(), 1);
        assert_eq!(rx.recv().await.unwrap(), 2);
    });
}
