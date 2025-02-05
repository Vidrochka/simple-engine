#![feature(iter_array_chunks)]

use std::{thread, time::Duration};

use ahash::AHashSet;
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

use systems::{debug::{DebugEndLayer, DebugStartLayer, DrawShapeLayer}, input::{BaseDeviceType, DeviceEvent, DeviceTypeDescription, DeviceTypeDescriptionBuilder, InputReadLayer, InputSystem}, render::{IRenderDependencies, RenderLayers, RenderState}, ui::UITestLayer};
use simple_layers::{layer::LayersStack, ILayersSystemDependencies};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use window::{device::DeviceCache, WindowCollection};
use winit::{application::ApplicationHandler, event::{ElementState, MouseScrollDelta, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, keyboard::PhysicalKey, window::WindowAttributes};
use xdi::{builder::DiBuilder, types::error::ServiceBuildResult, ServiceProvider};

pub mod systems;
pub mod window;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run() -> Result<(), impl std::error::Error> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch="wasm32")] {
            tracing_wasm::set_as_global_default();

            console_error_panic_hook::set_once();
        } else {
            let subscriber = FmtSubscriber::builder()
                .with_max_level(Level::INFO)
                .finish();

            tracing::subscriber::set_global_default(subscriber)
                .expect("Tracing subscriber registration error");
        }
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .build()
        .expect("Tokio rt build error");

    let di_builder = DiBuilder::new();

    let handle = rt.handle().clone();
    di_builder.singletone(move |_| Ok(handle.clone()));

    di_builder.register_layers_system_dependencies();
    di_builder.register_render_dependencies();
    // di_builder.register_device_system_dependencies();
    di_builder.singletone(InputSystem::new);
    di_builder.transient(DeviceCache::new);

    di_builder.thread_local(WindowCollection::new);
    di_builder.transient(SimpleEngineApp::new);

    let sp = di_builder.build();
    
    let mut app = sp.resolve::<SimpleEngineApp>().expect("Engine app not registered");

    let event_loop = EventLoop::new().expect("Event loop creation error");

    cfg_if::cfg_if! {
        if #[cfg(target_arch="wasm32")] {
            let res = event_loop.spawn(app);
        } else {
            let res = event_loop.run_app(&mut app);
        }
    };

    rt.shutdown_timeout(Duration::from_secs(2));

    res
}

#[derive(Debug)]
pub struct SimpleEngineApp {
    window_collection: WindowCollection,
    render_state: RenderState,
    input_system: InputSystem,

    layers_stack: LayersStack,

    device_cache: DeviceCache,
}

impl SimpleEngineApp {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        let mut layers_stack = sp.resolve::<LayersStack>()?;

        layers_stack.push_layer("debug_start", |_| Ok(DebugStartLayer::new()));

        layers_stack.push_layer("input_read", |sp| Ok(InputReadLayer::new(sp)?));

        layers_stack.push_layer("debug_shape", |sp| Ok(DrawShapeLayer::new(sp)?)).disable();

        layers_stack.push_layer("ui_test", |sp| Ok(UITestLayer::new(sp)?));

        layers_stack.register_source::<RenderLayers>();
    
        layers_stack.push_layer("debug_end", |sp| Ok(DebugEndLayer::new(sp)?));

        let input_system = sp.resolve::<InputSystem>()?;

        input_system.register_device_type(DeviceTypeDescriptionBuilder::default().with_ty(BaseDeviceType::Keyboard).with_description("Default keyboard").build().unwrap());
        input_system.register_device_type(DeviceTypeDescriptionBuilder::default().with_ty(BaseDeviceType::Mouse).with_description("Default mouse").build().unwrap());

        Ok(Self {
            window_collection: sp.resolve()?,
            render_state: sp.resolve()?,
            input_system,
            device_cache: sp.resolve()?,
            layers_stack,
        })
    }
}

impl ApplicationHandler for SimpleEngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {   
        event_loop.set_control_flow(ControlFlow::Poll);

        if !self.window_collection.has_main_window() {
            let window_attributes = WindowAttributes::default().with_title(
                "simpe engine",
            );
    
            let window = event_loop.create_window(window_attributes).expect("Window creation error");

            self.window_collection.add_window(window);
        }
    }
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                tracing::debug!("Redraw requested");
            },
            WindowEvent::Resized(size) => {
                if let Some(render_state) = &mut*self.render_state.get_mut() {
                    render_state.resize([size.width, size.height]);
                }
            },
            WindowEvent::MouseInput { device_id, state, button } => {
                let (mouse_id, event_channel) = self.device_cache.add_device(device_id, BaseDeviceType::Mouse);

                match state {
                    ElementState::Pressed => event_channel.send(DeviceEvent::ButtonDown { key: format!("{button:?}").into() }).expect("Device event send error"),
                    ElementState::Released => event_channel.send(DeviceEvent::ButtonUp { key: format!("{button:?}").into() }).expect("Device event send error"),
                }
            },
            WindowEvent::MouseWheel { device_id, delta, .. } => {
                let (mouse_id, event_channel) = self.device_cache.add_device(device_id, BaseDeviceType::Mouse);

                match delta {
                    MouseScrollDelta::LineDelta(x, y) => event_channel.send(DeviceEvent::PointerMove { point: [x as f64, y as f64].into() }).expect("Device event send error"),
                    MouseScrollDelta::PixelDelta(physical_position) => event_channel.send(DeviceEvent::PointerMove { point: [physical_position.x, physical_position.y].into() }).expect("Device event send error"),
                }
            }
            WindowEvent::KeyboardInput { device_id, event, .. } => {
                let (keyboard_id, event_channel) = self.device_cache.add_device(device_id, BaseDeviceType::Keyboard);

                if let PhysicalKey::Code(code)  = event.physical_key {
                    match event.state {
                        ElementState::Pressed => event_channel.send(DeviceEvent::ButtonDown { key: format!("{code:?}").into() }).expect("Device event send error"),
                        ElementState::Released => event_channel.send(DeviceEvent::ButtonUp { key: format!("{code:?}").into() }).expect("Device event send error"),
                    }
                }
            },
            WindowEvent::CursorMoved { device_id, position } => {
                let (mouse_id, event_channel) = self.device_cache.add_device(device_id, BaseDeviceType::Mouse);

                event_channel.send(DeviceEvent::PointerMove { point: [position.x, position.y].into() }).expect("Device event send error")
            },
            e => {
                tracing::info!("{e:?}");
                // thread::sleep(Duration::from_secs(1));
            }
        }   
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.window_collection.free();
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            winit::event::DeviceEvent::Removed => {
                let (_local_device_id, _ty) = self.device_cache.remove_device(&device_id);
            },
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.layers_stack.update();
        self.window_collection.get_window().expect("Base window not registered").request_redraw();
    }
}