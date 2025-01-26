use layer::{LayerCtx, LayersStack};
use scheduler::LayerScheduler;
use xdi::builder::DiBuilder;

pub mod layer;
pub mod scheduler;
pub mod types;

#[cfg(test)]
pub mod tests;

pub trait ILayersSystemDependencies {
    fn register_layers_system_dependencies(&self);
}

impl ILayersSystemDependencies for DiBuilder {
    fn register_layers_system_dependencies(&self) {
        self.thread_local(|_| Ok(LayerCtx::default()));
        self.transient(LayerScheduler::new);
        self.transient(LayersStack::new);
    }
}
