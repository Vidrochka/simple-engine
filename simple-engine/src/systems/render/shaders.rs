use std::{fs::File, io::Read, sync::Arc};

use dashmap::DashMap;
use wgpu::ShaderModule;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use super::RenderState;


#[derive(Debug, Clone)]
pub struct ShaderManager {
    render_state: RenderState,

    shaders: Arc<DashMap<String, Arc<ShaderModule>, ahash::RandomState>>,
}

impl ShaderManager {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            shaders: Default::default(),
            render_state: sp.resolve::<RenderState>()?,
        })
    }

    pub fn load_shader(&self, name: impl Into<String>, path: &str) {
        let mut file = File::open(path).unwrap();

        let mut file_data = String::new(); 
        file.read_to_string(&mut file_data).unwrap();
        
        let shader = self.render_state.get().as_ref().unwrap().device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(file_data.into()),
        });

        self.shaders.insert(name.into(), Arc::new(shader));
    }

    pub fn has_shader(&self, name: impl AsRef<str>) -> bool {
        self.shaders.contains_key(name.as_ref())
    }

    pub fn get_shader(&self, name: impl AsRef<str>) -> Option<Arc<ShaderModule>> {
        self.shaders.get(name.as_ref()).as_deref().cloned()
    }
}