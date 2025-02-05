use std::collections::HashMap;

use crossbeam_channel::Sender;
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use winit::event::DeviceId as WinitDeviceId;
use crate::systems::input::{DeviceEvent, DeviceId, DeviceType, InputSystem};


/// Буффер id девайсов
/// 
/// Т.к. winit не предоставляет список активных девайсов, в момент получения события мы регистрируем новый девайс через кеш девайсов.
/// Если девайс уже зарегистрирован, то возвращаем его id, иначе регистрируем новый девайс и возвращаем его id.
#[derive(Debug)]
pub struct DeviceCache {
    device_list: HashMap<WinitDeviceId, (DeviceId, Sender<DeviceEvent>), ahash::RandomState>,

    input_system: InputSystem,
}

impl DeviceCache {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            device_list: Default::default(),
            input_system: sp.resolve()?,
        })
    }

    pub fn add_device(&mut self, id: WinitDeviceId, ty: impl Into<DeviceType>) -> &(DeviceId, Sender<DeviceEvent>) {
        self.device_list.entry(id)
            .or_insert_with(|| self.input_system.register_device(ty.into()))
    }

    pub fn remove_device(&mut self, id: &WinitDeviceId) -> (Option<DeviceId>, Option<DeviceType>) {
        let Some((local_id, _)) = self.device_list.remove(id) else {
            return (None, None);
        };

        let ty = self.input_system.remove_device(&local_id);

        (Some(local_id), ty)
    }
}

