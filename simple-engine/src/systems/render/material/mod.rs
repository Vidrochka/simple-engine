use std::{collections::HashMap, sync::Arc};

use cache::MaterialsCache;
use mint::Vector4;
use parking_lot::RwLock;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

pub mod cache;

#[derive(Debug, Clone)]
pub struct MaterialSystem {
    inner: Arc<RwLock<MaterialSystemInner>>,
}

#[derive(Debug, Default)]
pub struct MaterialSystemInner {
    materials: HashMap<String, Arc<Material>>,
    cache: MaterialsCache,
}

impl MaterialSystem {
    pub fn new(_sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            inner: Default::default()
        })
    }

    pub fn add_material(&self, material: Material) -> String {
        let mut inner = self.inner.write();

        let name = inner.cache.add_material(&material);

        if !inner.materials.contains_key(&name) {
            tracing::info!("Add material {material:?}");
            inner.materials.insert(name.clone(), Arc::new(material));
        }

        name
    }

    pub fn add_material_with_name(&self, name: impl Into<String>, material: Material) {
        let name = name.into();
        let mut inner = self.inner.write();

        inner.materials.insert(name.clone(), Arc::new(material));
    }

    pub fn get_material(&self, name: impl Into<String>) -> Option<Arc<Material>> {
        self.inner.read().materials.get(&name.into()).cloned()
    }

    pub fn count(&self) -> usize {
        self.inner.read().materials.len()
    }
}

#[derive(Debug, Hash)]
pub struct Material {
    pub color: Vector4<u8>,
}