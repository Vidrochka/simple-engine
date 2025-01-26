use std::sync::Arc;

use mint::Vector3;
use parking_lot::{Mutex, MutexGuard};
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

#[derive(Debug, Clone)]
pub struct RenderCommandsManager {
    buffers: Arc<Mutex<Vec<RenderCommand>>>
}

impl RenderCommandsManager {
    pub fn new(_sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self { buffers: Default::default() })
    }

    pub fn add_buffer(&self, buffer: RenderCommandBuffer) {
        self.buffers.lock().extend(buffer.commands);
    }

    pub fn get_mut<'a>(&'a self) -> MutexGuard<'a, Vec<RenderCommand>> {
        self.buffers.lock()
    }
}

pub struct RenderCommandBuffer {
    commands: Vec<RenderCommand>,
}

impl RenderCommandBuffer {
    pub fn new() -> Self {
        Self { commands: Default::default() }
    }

    pub fn push(&mut self, command: RenderCommand) {
        self.commands.push(command);
    } 
}

pub type VertexList = Vec<Vector3<f32>>;
pub type IndexList = Vec<u16>;

#[derive(Debug)]
pub enum RenderCommand {
    Shape {
        material: String,
        vertex: VertexList,
        index: Option<IndexList>,
    }
}

impl RenderCommand {
    pub fn shape(name: impl Into<String>, vertex: impl Into<VertexList>) -> RenderCommand {
        RenderCommand::Shape { material: name.into(), vertex: vertex.into(), index: None }
    }

    pub fn indexed_shape(name: impl Into<String>, vertex: impl Into<VertexList>, index: impl Into<IndexList>) -> RenderCommand {
        RenderCommand::Shape { material: name.into(), vertex: vertex.into(), index: Some(index.into()) }
    }
}