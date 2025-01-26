use std::{collections::HashMap, fmt::Debug, sync::Arc};

use chrono::{DateTime, TimeDelta, Utc};
use parking_lot::RwLock;
use xdi::{ServiceProvider, types::error::ServiceBuildResult};

use crate::{
    scheduler::LayerScheduler,
    types::{
        id::LayerId,
        type_info::{TypeInfo, TypeInfoSource},
    },
};

#[derive(Debug)]
pub struct LayersStack {
    scheduler: LayerScheduler,

    layers_order: Vec<LayerId>,
    layers_map: HashMap<LayerId, Layer, ahash::RandomState>,
    layer_name_to_id: HashMap<String, LayerId, ahash::RandomState>,

    last_update: Option<DateTime<Utc>>,

    sp: ServiceProvider,
}

impl LayersStack {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            scheduler: sp.resolve()?,
            layers_order: Default::default(),
            layers_map: Default::default(),
            layer_name_to_id: Default::default(),
            last_update: None,
            sp,
        })
    }

    pub fn push_layer<
        TLayer: ILayer + 'static,
        TCtr: Fn(ServiceProvider) -> anyhow::Result<TLayer> + 'static,
    >(
        &mut self,
        name: impl Into<String>,
        layer_ctr: TCtr,
    ) -> &mut Layer {
        let name = name.into();

        let layer = Layer::new(name.clone(), layer_ctr, true);

        let layer_id = layer.id();

        self.layers_order.push(layer_id);
        self.layers_map.insert(layer_id, layer);
        self.layer_name_to_id.insert(name, layer_id);

        self.layers_map.get_mut(&layer_id).unwrap()
    }

    pub fn register_source<TLayersSource: ILayersSource>(&mut self) {
        TLayersSource::register(self);
    }

    pub fn update(&mut self) {
        let last_update = self.last_update.unwrap_or_else(|| Utc::now());
        let now = Utc::now();

        let dt = now - last_update;

        for layer_id in &self.layers_order {
            // TODO: add err handling
            let layer = self.layers_map.get_mut(&layer_id).unwrap();
            layer
                .update(&self.sp, &dt, &mut self.scheduler)
                .inspect_err(|e| tracing::error!("{e:?}"))
                .unwrap();
        }

        self.scheduler.wait_all_blocking();

        self.last_update = Some(now);
    }

    pub fn enable(&mut self, id: LayerId) {
        if let Some(layer) = self.layers_map.get_mut(&id) {
            layer.enable();
        }
    }

    pub fn disable(&mut self, id: LayerId) {
        if let Some(layer) = self.layers_map.get_mut(&id) {
            layer.disable();
        }
    }
}

pub trait ILayer
where
    Self: Debug,
{
    #[allow(unused)]
    fn on_update(&mut self, dt: &TimeDelta, scheduler: &mut LayerScheduler) {}
}

pub trait ILayersSource {
    fn register(layers_stack: &mut LayersStack);
}

#[derive(Debug)]
pub struct Layer {
    ctr: LayerBuilder,
    state: LayerState,
    enabled: bool,

    ty: TypeInfo,
    name: String,
    id: LayerId,
}

impl Layer {
    pub fn new<
        TLayer: ILayer + 'static,
        TCtr: Fn(ServiceProvider) -> anyhow::Result<TLayer> + 'static,
    >(
        name: String,
        layer_ctr: TCtr,
        enabled: bool,
    ) -> Self {
        Self {
            ctr: LayerBuilder(Box::new(move |sp| {
                let layer = (layer_ctr)(sp)?;
                Ok(Box::new(layer))
            })),
            state: LayerState::Pending,
            enabled,
            ty: TLayer::type_info(),
            name,
            id: LayerId::new(),
        }
    }

    pub fn enable(&mut self) -> &mut Self {
        self.enabled = true;
        self
    }

    pub fn disable(&mut self) -> &mut Self {
        self.enabled = false;
        self
    }

    pub fn update(
        &mut self,
        sp: &ServiceProvider,
        dt: &TimeDelta,
        scheduler: &mut LayerScheduler,
    ) -> anyhow::Result<bool> {
        if !self.enabled {
            return Ok(false);
        }

        let ctx = sp.resolve::<LayerCtx>().unwrap();

        ctx.change(LayerCtxInner {
            id: self.id,
            name: self.name.clone(),
        });

        match &mut self.state {
            LayerState::Pending => {
                tracing::debug!("[{name}] <{id}> Layer first update", id = self.id, name = self.name);

                let mut service = self.ctr.build(sp.clone())?;

                tracing::debug!("[{name}] <{id}> Layer created", id = self.id, name = self.name);

                service.on_update(dt, scheduler);

                tracing::debug!("[{name}] <{id}> Layer updated", id = self.id, name = self.name);

                self.state = LayerState::Created { service };
            }
            LayerState::Created { service } => {
                tracing::debug!("[{name}] <{id}> Layer update", id = self.id, name = self.name);

                service.on_update(dt, scheduler);

                tracing::debug!("[{name}] <{id}> Layer updated", id = self.id, name = self.name);
            }
        }

        Ok(true)
    }

    pub fn ty(&self) -> TypeInfo {
        self.ty
    }

    pub fn id(&self) -> LayerId {
        self.id
    }
}

#[derive(Debug)]
pub enum LayerState {
    Pending,
    Created { service: Box<dyn ILayer> },
}

pub struct LayerBuilder(Box<dyn Fn(ServiceProvider) -> anyhow::Result<Box<dyn ILayer>>>);

impl Debug for LayerBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LayerBuilder").finish()
    }
}

impl LayerBuilder {
    fn build(&self, sp: ServiceProvider) -> anyhow::Result<Box<dyn ILayer>> {
        (self.0)(sp)
    }
}

#[derive(Clone, Default)]
pub struct LayerCtx {
    inner: Arc<RwLock<LayerCtxInner>>,
}

impl LayerCtx {
    pub fn id(&self) -> LayerId {
        self.inner.read().id
    }

    pub fn name(&self) -> String {
        self.inner.read().name.clone()
    }

    pub(crate) fn change(&self, ctx: LayerCtxInner) {
        *self.inner.write() = ctx;
    }
}

#[derive(Debug)]
pub struct LayerCtxInner {
    id: LayerId,
    name: String,
}

impl Default for LayerCtxInner {
    fn default() -> Self {
        Self {
            id: LayerId::null(),
            name: "".to_string(),
        }
    }
}
