use std::{thread, time::Duration};

use mint::Vector2;
use simple_ui::{layers::*, render::command::{UIViewRender, UIRenderCommand}, style::FillStyleBuilder, view::*};
use tracing::{error, warn, Level};
use tracing_subscriber::FmtSubscriber;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowAttributes},
};

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::WARN)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
    
    let event_loop = EventLoop::new().unwrap();

    #[allow(deprecated)]
    let window = event_loop.create_window(WindowAttributes::default().with_title(
        "Render layers",
    )).unwrap();

    let mut state = State::new(&window).await;

    #[allow(deprecated)]
    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            }  if window_id == state.window().id() => if !state.input(event)
            {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        // This tells winit that we want another frame after this one
                        state.window().request_redraw();
            
                        // if !surface_configured {
                        //     return;
                        // }
            
                        state.update();

                        let commands = build_commands(Vector2::from_slice(&[state.size.width as f32, state.size.height as f32]));

                        match state.render(commands) {
                            Ok(_) => {}
                            // Reconfigure the surface if it's lost or outdated
                            Err(
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                            ) => state.resize(state.size),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                error!("OutOfMemory");
                                control_flow.exit();
                            }
            
                            // This happens when the a frame takes too long to present
                            Err(wgpu::SurfaceError::Timeout) => {
                                warn!("Surface timeout")
                            }
                        }

                        // tracing::warn!("------------------------------------------------------------------------------------------------------------------");
                        // thread::sleep(Duration::from_secs(2));
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }).unwrap();
}


struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,

    render_pipeline: wgpu::RenderPipeline,
    depth_texture: Texture,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None, // 6.
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            depth_texture,
        }
    }
    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }


    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
    }

    fn render(&mut self, commands: Vec<UIRenderCommand>) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            

            for (idx, command) in commands.iter().enumerate() {
                // tracing::warn!("{command:?}");

                let indexes = command.triangulate().unwrap();

                let index_buffer = self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&indexes),
                        usage: wgpu::BufferUsages::INDEX,
                    }
                );
                
                let vertexes = match command {
                    UIRenderCommand::DrawShape { points, color, .. } => {
                        points.into_iter().map(|point| Vertex {
                            position: [point.x * 2f32 - 1f32, 1f32 - point.y * 2f32, 0.0],
                            color: [color.x as f32 / 255f32, color.y as f32 / 255f32, color.z as f32 / 255f32],
                        }).collect::<Vec<_>>()
                    },
                };

                // tracing::error!("{vertexes:?}");

                let vertex_buffer = self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertexes),
                        usage: wgpu::BufferUsages::VERTEX,
                    }
                );

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..indexes.len() as u32, 0, 0..1);
            }
        }
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())    
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
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

fn build_commands(size: Vector2<f32>) -> Vec<UIRenderCommand> {
    let mut view = View::new("Test view", size);

    let mut partial_view = PartialView::new();

    partial_view.add_layer(Layer::Shape(ShapesLayerBuilder::default()
        .id("1")
        .name("Shape layer 1")
        .shapes(vec![
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([40, 40])
                    .position([10, 40])
                    .fill(FillStyleBuilder::default().color([20, 20, 20]).build().unwrap())
                    .build()
                    .unwrap()
            ),
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([60, 60])
                    .position([2, 12])
                    .fill(FillStyleBuilder::default().color([0, 255, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
        ])
        .build()
        .unwrap())
    );

    let id = partial_view.add_layer(Layer::Flex(FlexLayerBuilder::default()
        .id("2")
        .name("Flex layer 2")
        .direction(FlexDirection::Vertical)
        // .fill(FillStyleBuilder::default().color([0, 0, 0, 255]).build().unwrap())
        .gap(10u16)
        .build()
        .unwrap())
    );

    partial_view.add_child_layer(&id, Layer::Shape(ShapesLayerBuilder::default().id("3")
        .name("Shape layer 2")
        .shapes(vec![
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([40, 40])
                    .position([30, 2])
                    .fill(FillStyleBuilder::default().color([0, 255, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([70, 70])
                    .position([14, 2])
                    .fill(FillStyleBuilder::default().color([0, 0, 255]).build().unwrap())
                    .build()
                    .unwrap()
            ),
        ])
        .build()
        .unwrap()
    ));

    partial_view.add_child_layer(&id, Layer::Shape(ShapesLayerBuilder::default().id("4")
        .name("Shape layer 3")
        .shapes(vec![
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([200, 200])
                    .position([2, 2])
                    .fill(FillStyleBuilder::default().color([0, 255, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([300, 300])
                    .position([14, 2])
                    .fill(FillStyleBuilder::default().color([255, 0, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
        ])
        .build()
        .unwrap()
    ));

    let id = view.add_view_layer(Layer::Flex(FlexLayerBuilder::default()
        .id("5")
        .name("Flex layer")
        .gap(10u16)
        // .fill(FillStyleBuilder::default().color([0, 0, 255, 0]).build().unwrap())
        .build()
        .unwrap())
    );

    view.replace_child_layers(id, partial_view);

    let render = UIViewRender::new();

    let commands = render.write_view(&view);

    commands
}

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.
    
    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum CompareFunction {
    Undefined = 0,
    Never = 1,
    Less = 2,
    Equal = 3,
    LessEqual = 4,
    Greater = 5,
    NotEqual = 6,
    GreaterEqual = 7,
    Always = 8,
}