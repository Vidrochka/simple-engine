use simple_layers::layer::ILayer;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use crate::systems::input::InputSystem;


#[derive(Debug)]
pub struct InputReadLayer {
    input_system: InputSystem,
}

impl InputReadLayer {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            input_system: sp.resolve()?,
        })
    }
}

impl ILayer for InputReadLayer {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, scheduler: &mut simple_layers::scheduler::LayerScheduler) {
        let input_system = self.input_system.clone();

        scheduler.schedule(async move {
            input_system.flush_events();

            let all_events = input_system.get_events();

            if all_events.is_empty() {
                return;
            }

            tracing::info!("All events: {:?}", all_events);
        }, ());
    }
}