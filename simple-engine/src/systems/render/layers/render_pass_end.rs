use simple_layers::layer::ILayer;
use xdi::{ServiceProvider, types::error::ServiceBuildResult};

use crate::systems::render::{FrameOutputState, RenderState};

#[derive(Debug)]
pub struct RenderPassEndLayer {
    render_state: RenderState,
    output_state: FrameOutputState,
}

impl RenderPassEndLayer {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            render_state: sp.resolve()?,
            output_state: sp.resolve()?,
        })
    }
}

impl ILayer for RenderPassEndLayer {
    fn on_update(
        &mut self,
        _dt: &chrono::TimeDelta,
        scheduler: &mut simple_layers::scheduler::LayerScheduler,
    ) {
        let render_state = self.render_state.clone();
        let output_state= self.output_state.clone();

        scheduler.schedule(async move {
            let render_state = render_state.get();

            let Some(render_state) = &*render_state else {
                return;
            };

            let mut output_state = output_state.get_mut();

            let Some(mut output_state) = output_state.take() else {
                return;
            };

            output_state.complete_render_pass();
            output_state.complete_frame(render_state);
        }, ["render_pass_start"]);
    }
}
