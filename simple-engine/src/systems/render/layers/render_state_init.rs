use simple_layers::layer::ILayer;
use tokio::runtime::Handle;
use wgpu::SurfaceTargetUnsafe;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use crate::{systems::render::{RenderState, RenderStateInner}, window::WindowCollection};


#[derive(Debug)]
pub struct RenderStateInitLayer {
    render_state: RenderState,

    window_collection: WindowCollection,
    rt: Handle,
}

impl RenderStateInitLayer {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            render_state: sp.resolve()?,
            window_collection: sp.resolve()?,
            rt: sp.resolve()?,
        })
    }
}

impl ILayer for RenderStateInitLayer {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, _scheduler: &mut simple_layers::scheduler::LayerScheduler) {
        let Some(window) = self.window_collection.get_window() else {
            return;
        };

        let mut state = self.render_state.get_mut();

        if let Some(state) = &mut *state {
            state.resize(window.size());

            return;
        }


        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::from_window(&*window.get_ref()).unwrap()).unwrap()
        };

        let (adapter, device, queue) = self.rt.block_on(async {
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
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            ).await.unwrap();

            (adapter, device, queue)
        });

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let window_size = window.size();

        assert!(window_size.x > 0);
        assert!(window_size.y > 0);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.x,
            height: window_size.y,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);
        
        *state = Some(RenderStateInner {
            instance,
            surface,
            device,
            queue,
            config,
            size: window_size,
        })
    }
}