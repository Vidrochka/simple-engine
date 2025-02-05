use crossbeam_channel::{unbounded, Receiver, Sender};
use derive_builder::Builder;
use mint::{Vector2, Vector3};
use strum::{AsRefStr, EnumString};
use uuid::Uuid;




#[derive(Debug, Builder)]
#[builder(setter(into, prefix = "with"))]
pub struct DeviceTypeDescription {
    ty: DeviceType,

    #[builder(default)]
    description: String,

    //TODO: add more characteristics
}

impl DeviceTypeDescription {
    pub fn ty(&self) -> &DeviceType {
        &self.ty
    }
    
    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct DeviceId(Uuid);

impl DeviceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceType(String);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumString, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum BaseDeviceType {
    Keyboard,
    Mouse,
    Gamepad,
    Touchscreen,
}

impl Into<DeviceType> for BaseDeviceType {
    fn into(self) -> DeviceType {
        DeviceType(self.as_ref().to_string())
    }
}

impl Into<DeviceType> for String {
    fn into(self) -> DeviceType {
        DeviceType(self)
    }
}

#[derive(Debug)]
pub struct DeviceDescriptor {
    id: DeviceId,
    sender: Sender<DeviceEvent>,
    receiver: Receiver<DeviceEvent>,

    last_frame_events_buffer: Vec<DeviceEvent>,
}

impl DeviceDescriptor {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();

        Self {
            id: DeviceId::new(),
            sender,
            receiver,
            last_frame_events_buffer: Default::default()
        }
    }
    
    pub fn id(&self) -> &DeviceId {
        &self.id
    }
    
    pub fn sender(&self) -> &Sender<DeviceEvent> {
        &self.sender
    }
    
    pub fn receiver(&self) -> &Receiver<DeviceEvent> {
        &self.receiver
    }

    pub fn flush_events(&mut self) {
        self.last_frame_events_buffer.clear();

        self.receiver.try_iter().for_each(|x| self.last_frame_events_buffer.push(x));
    }
    
    pub fn last_frame_events_buffer(&self) -> &[DeviceEvent] {
        &self.last_frame_events_buffer
    }
}

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    ButtonDown {
        key: ButtonType,
    },
    ButtonUp {
        key: ButtonType,
    },
    PointerMove {
        point: Vector2<f64>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ButtonType(String);

#[derive(Debug, PartialEq, Eq, Hash, EnumString, AsRefStr)]
pub enum BaseButtonType {

}

impl Into<ButtonType> for BaseButtonType {
    fn into(self) -> ButtonType {
        ButtonType(self.as_ref().to_string())
    }
}

impl Into<ButtonType> for String {
    fn into(self) -> ButtonType {
        ButtonType(self)
    }
}