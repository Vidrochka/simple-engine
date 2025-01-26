use std::sync::Arc;

use mint::Vector2;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use wgpu::{Device, Instance, Queue, Surface, SurfaceConfiguration};
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use super::frame_output_state::FrameOutputStateInner;


#[derive(Debug, Clone)]
pub struct RenderState {
    inner: Arc<RwLock<Option<RenderStateInner>>>,
}

impl RenderState {
    pub fn new(_: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self { inner: Default::default() })
    }

    pub fn get(&self) -> RwLockReadGuard<'_, Option<RenderStateInner>> {
        self.inner.read()
    }

    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Option<RenderStateInner>> {
        self.inner.write()
    }
}

#[derive(Debug)]
pub struct RenderStateInner {
    #[allow(unused)]
    pub (crate) instance: Instance,
    pub (crate) surface: Surface<'static>,
    pub (crate) device: Device,
    pub (crate) queue: Queue,
    pub (crate) config: SurfaceConfiguration,
    pub (crate) size: Vector2<u32>,
}

impl RenderStateInner {
    pub fn new_frame(&self) -> FrameOutputStateInner {
        FrameOutputStateInner::new(self)
    }

    pub fn resize(&mut self, size: impl Into<Vector2<u32>>) {
        let size = size.into();

        self.size = size;

        self.config.width = self.size.x;
        self.config.height = self.size.y;

        self.surface.configure(&self.device, &self.config);
    }
}