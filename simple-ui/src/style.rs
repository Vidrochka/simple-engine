use derive_builder::Builder;
use mint::Vector3;
use serde::Serialize;

#[derive(Debug, Serialize, Clone, Builder)]
#[builder(try_setter, setter(into))]
pub struct FillStyle {
    #[serde(rename = "c")]
    pub color: Vector3<u8>,
}

impl Default for FillStyle {
    fn default() -> Self {
        Self { color: Vector3::from([0, 0, 0]) }
    }
}

#[derive(Debug, Hash)]
pub struct UIMaterial {
    pub color: Vector3<u8>,
}