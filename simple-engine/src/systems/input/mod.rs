pub mod device;
pub use device::*;

pub mod layers;
pub use layers::*;

use std::{collections::HashMap, sync::Arc};

use crossbeam_channel::Sender;
use parking_lot::RwLock;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};


#[derive(Debug, Clone)]
pub struct InputSystem {
    device_types: Arc<RwLock<HashMap<DeviceType, DeviceTypeDescription, ahash::RandomState>>>,

    devices: Arc<RwLock<HashMap<DeviceType, Vec<DeviceDescriptor>, ahash::RandomState>>>,
}

impl InputSystem {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            device_types: Default::default(),
            devices: Default::default()
        })
    }

    /// Registers new device type
    pub fn register_device_type(&self, description: DeviceTypeDescription) {
        self.device_types.write().insert(description.ty().clone(), description);
    }

    /// Registers new device and returns its id
    pub fn register_device(&self, ty: DeviceType) -> (DeviceId, Sender<DeviceEvent>) {
        let descriptor = DeviceDescriptor::new();
        let id = *descriptor.id();
        let event_source = descriptor.sender().clone();

        self.devices.write().entry(ty).or_default().push(descriptor);

        (id, event_source)
    }

    /// Returns device type by its id
    pub fn get_device_type(&self, id: &DeviceId) -> Option<DeviceType> {
        self.devices.read().iter().find_map(|(ty, descriptiors)| {
            if descriptiors.iter().any(|x| x.id() == id) {
                Some(ty.clone())
            } else {
                None
            }
        })
    }

    /// Removes device from the system and returns its type
    pub fn remove_device(&mut self, id: &DeviceId) -> Option<DeviceType> {
        self.devices.write().iter_mut().find_map(|(ty, ids)| {
            if let Some(index) = ids.iter().position(|x| x.id() == id) {
                ids.remove(index);
                Some(ty.clone())
            } else {
                None
            }
        })
    }
    
    pub fn flush_events(&self) {
        self.devices.write().values_mut().for_each(|x|
            x.iter_mut().for_each(|y| y.flush_events())
        );
    }

    pub fn get_events(&self) -> Vec<DeviceEvent> {
        self.devices.read().values().flat_map(|x| x.iter().flat_map(|dd| dd.last_frame_events_buffer().iter().cloned())).collect()
    }
}