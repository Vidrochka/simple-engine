use simple_layers::layer::ILayer;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use crate::systems::render::RenderPipelineManager;


#[derive(Debug)]
pub struct RenderPipelineInitLayer {
    render_pipeline_manager: RenderPipelineManager,
}

impl RenderPipelineInitLayer {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            render_pipeline_manager: sp.resolve()?,
        })
    }
}

impl ILayer for RenderPipelineInitLayer {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, scheduler: &mut simple_layers::scheduler::LayerScheduler) {
        let render_pipeline_manager = self.render_pipeline_manager.clone();

        scheduler.schedule(async move {
            if !render_pipeline_manager.has_pipeline("default") {
                render_pipeline_manager.add_pipeline("default");
            }
        }, ["shader_init_layer"]);
    }
}