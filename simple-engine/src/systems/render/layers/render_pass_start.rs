use simple_layers::layer::ILayer;
use xdi::{ServiceProvider, types::error::ServiceBuildResult};

use crate::systems::render::{FrameOutputState, RenderState};

#[derive(Debug)]
pub struct RenderPassStartLayer {
    render_state: RenderState,
    output_state: FrameOutputState,
}

impl RenderPassStartLayer {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            render_state: sp.resolve()?,
            output_state: sp.resolve()?,
        })
    }
}

impl ILayer for RenderPassStartLayer {
    fn on_update(
        &mut self,
        _dt: &chrono::TimeDelta,
        scheduler: &mut simple_layers::scheduler::LayerScheduler,
    ) {
        let render_state = self.render_state.clone();
        let output_state = self.output_state.clone();

        scheduler.schedule(async move {
            let render_state = render_state.get();

            let Some(render_state) = &*render_state else {
                return;
            };
    
            let mut output_state = output_state.get_mut();
    
            if let Some(output_state) = &mut *output_state {
                output_state.new_render_pass([ 0.1, 0.2, 0.3, 1.0]);
            } else {
                let mut output = render_state.new_frame();
                
                output.new_render_pass([ 0.1, 0.2, 0.3, 1.0]);
    
                *output_state = Some(output);
            }
        }, ["render_pipeline_init"]);
    }
}
