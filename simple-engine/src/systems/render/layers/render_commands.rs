use simple_layers::layer::ILayer;
use wgpu::util::DeviceExt;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use crate::systems::render::{FrameOutputState, MaterialSystem, RenderCommand, RenderCommandsManager, RenderPipelineManager, RenderState, Vertex};


#[derive(Debug)]
pub struct RenderCommansLayer {
    render_command_manager: RenderCommandsManager,
    render_pipeline_manager: RenderPipelineManager,
    material_system: MaterialSystem,

    output: FrameOutputState,
    render_state: RenderState,
}

impl RenderCommansLayer {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            render_command_manager: sp.resolve()?,
            render_pipeline_manager: sp.resolve()?,
            material_system: sp.resolve()?,
            output: sp.resolve()?,
            render_state: sp.resolve()?,
        })
    }
}

impl ILayer for RenderCommansLayer {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, scheduler: &mut simple_layers::scheduler::LayerScheduler) {
        scheduler.wait_all_blocking();

        self.render_pipeline_manager.enable_render_pipeline("default");

        let mut render_state_lock = self.render_state.get_mut();

        let render_state = render_state_lock.as_mut().unwrap();

        let mut output_lock = self.output.get_mut();

        let output = output_lock.as_mut().unwrap();

        let render_pass = output.render_pass.as_mut().unwrap();

        for command in self.render_command_manager.get_mut().drain(..) {
            // tracing::debug!("{command:?}");

            match command {
                RenderCommand::Shape { material, vertex, index } => {
                    let material = self.material_system.get_material(&material).unwrap();

                    let vertex = vertex.into_iter().map(|vertex| Vertex {
                        position: vertex.into(),
                        color: [material.color.x as f32 / 255.0, material.color.y as f32 / 255.0, material.color.z as f32 / 255.0]
                    }).collect::<Vec<_>>();

                    let vertex_buffer = render_state.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertex),
                            usage: wgpu::BufferUsages::VERTEX,
                        }
                    );

                    let index = index.unwrap();

                    let index_buffer = render_state.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&index),
                            usage: wgpu::BufferUsages::INDEX,
                        }
                    );

                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                    render_pass.draw_indexed(0..index.len() as u32, 0, 0..1);
                }
            };

        }
    }
}