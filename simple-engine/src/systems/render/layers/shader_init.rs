use simple_layers::layer::ILayer;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use crate::systems::render::ShaderManager;

// TODO: Вероятно следует сделать менеджер ресурсов, разные источники ресурсов и тут только указывать намерение грузить ресурс

#[derive(Debug)]
pub struct ShaderInitLayer {
    shader_collection: ShaderManager,
}

impl ShaderInitLayer {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            shader_collection: sp.resolve()?,
        })
    }
}

impl ILayer for ShaderInitLayer {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, scheduler: &mut simple_layers::scheduler::LayerScheduler) {
        let shader_collection = self.shader_collection.clone();

        scheduler.schedule(async move {
            if !shader_collection.has_shader("default") {
                shader_collection.load_shader("default", "./simple-engine/shaders/shader.wgsl");
            }
        }, ["render_state_init"]);
    }
}