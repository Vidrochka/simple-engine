pub mod shaders;
pub use shaders::*;

pub mod command;
pub use command::*;

pub mod material;
pub use material::*;

pub mod layers;
pub use layers::*;

pub mod render_state;
pub use render_state::*;

pub mod frame_output_state;
pub use frame_output_state::*;

pub mod render_pipeline;
pub use render_pipeline::*;

use xdi::builder::DiBuilder;
use simple_layers::layer::ILayersSource;

pub trait IRenderDependencies {
    fn register_render_dependencies(&self);
}

impl IRenderDependencies for DiBuilder {
    fn register_render_dependencies(&self) {
        self.singletone(RenderState::new);
        self.singletone(FrameOutputState::new);
        self.singletone(ShaderManager::new);
        self.singletone(RenderPipelineManager::new);
        self.singletone(MaterialSystem::new);
        self.singletone(RenderCommandsManager::new);
    }
}

pub struct RenderLayers;

impl ILayersSource for RenderLayers {
    fn register(layers_stack: &mut simple_layers::layer::LayersStack) {
        layers_stack.push_layer("render_state_init", |sp| Ok(RenderStateInitLayer::new(sp)?));
        layers_stack.push_layer("shader_init_layer", |sp| Ok(ShaderInitLayer::new(sp)?));
        layers_stack.push_layer("render_pipeline_init", |sp| Ok(RenderPipelineInitLayer::new(sp)?));

        layers_stack.push_layer("render_pass_start", |sp| Ok(RenderPassStartLayer::new(sp)?));

        layers_stack.push_layer("render_command", |sp| Ok(RenderCommansLayer::new(sp)?));

        layers_stack.push_layer("render_pass_end", |sp| Ok(RenderPassEndLayer::new(sp)?));
    }
}