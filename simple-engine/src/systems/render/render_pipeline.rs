use std::sync::Arc;

use dashmap::DashMap;
use wgpu::RenderPipeline;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use super::{FrameOutputState, RenderState, ShaderManager};


#[derive(Debug, Clone)]
pub struct RenderPipelineManager {
    render_state: RenderState,
    frame_output_state: FrameOutputState,

    shader_manager: ShaderManager,

    pipelines: Arc<DashMap<String, RenderPipeline, ahash::RandomState>>,
}

impl RenderPipelineManager {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            render_state: sp.resolve()?,
            frame_output_state: sp.resolve()?,
            shader_manager: sp.resolve()?,
            pipelines: Default::default()
        })
    }

    pub fn has_pipeline(&self, name: impl Into<String>) -> bool {
        self.pipelines.contains_key(&name.into())
    }

    pub fn add_pipeline(&self, name: impl Into<String>) {
        let name: String = name.into();

        let render_state_lock = self.render_state.get();

        let render_state = render_state_lock.as_ref().unwrap();

        let render_pipeline_layout = render_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&name),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let shader = self.shader_manager.get_shader("default").unwrap();

        let render_pipeline = render_state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_state.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        tracing::info!("Render pipeline with name [{name}] created");

        self.pipelines.insert(name, render_pipeline);
    }

    pub fn enable_render_pipeline(&self, pipeline_name: impl Into<String>) {
        let name: String = pipeline_name.into();

        tracing::debug!("Render pipeline with name [{name}] enabled");

        let pipeline = self.pipelines.get(&name).unwrap();

        let mut frame_output_state = self.frame_output_state.get_mut();

        frame_output_state.as_mut().unwrap().set_pipeline(pipeline.value());
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}