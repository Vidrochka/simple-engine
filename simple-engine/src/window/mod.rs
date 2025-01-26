use std::sync::Arc;

use mint::Vector2;
use parking_lot::{Mutex, MutexGuard};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};


#[derive(Debug, Clone)]
pub struct WindowCollection {
    windows: Arc<Mutex<Vec<SEWindow>>>,
}

impl WindowCollection {
    pub fn new(_sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            windows: Default::default(),
        })
    }

    pub fn has_main_window(&self) -> bool {
        !self.windows.lock().is_empty()
    }

    pub fn add_window(&self, window: Window) {
        self.windows.lock().push(SEWindow::new(window));
    }

    pub fn get_window(&self) -> Option<SEWindow> {
        self.windows.lock().first().cloned()
    }

    pub fn free(&self) {
        self.windows.lock().drain(..);
    }
}

#[derive(Debug, Clone)]
pub struct SEWindow {
    window: Arc<Mutex<Window>>
}

impl SEWindow {
    pub fn new(window: Window) -> Self {
        Self { window: Arc::new(Mutex::new(window)) }
    }

    pub fn request_redraw(&self) {
        self.window.lock().request_redraw();
    }

    pub fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.window.lock().display_handle().unwrap().as_raw()
    }

    pub fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.window.lock().window_handle().unwrap().as_raw()
    }

    pub fn size(&self) -> Vector2<u32> {
        let size = self.window.lock().inner_size();
        
        Vector2::from([
            size.width,
            size.height,
        ])
    }

    pub fn get_ref(&self) -> MutexGuard<Window> {
        self.window.lock()
    }
}