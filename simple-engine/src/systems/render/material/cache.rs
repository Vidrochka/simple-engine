use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use uuid::Uuid;

use super::Material;

#[derive(Debug, Default)]
pub struct MaterialsCache {
    materials: HashMap<u64, String, ahash::RandomState>,
}

impl MaterialsCache {
    pub fn new() -> Self {
        Self { materials: Default::default() }
    }

    pub fn add_material(&mut self, material: &Material) -> String {
        let mut s = DefaultHasher::new();
        material.hash(&mut s);
        let hash = s.finish();

        if let Some(name) = self.materials.get(&hash) {
            return name.clone();
        }
        
        let id = Uuid::new_v4();

        self.materials.insert(hash, id.to_string());

        id.to_string()
    }
}