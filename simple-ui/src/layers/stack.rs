use derive_builder::Builder;
use mint::Vector2;
use serde::Serialize;

use crate::layer_id::LayerId;


#[derive(Debug, Serialize, Clone, Builder)]
#[builder(try_setter, setter(into))]
pub struct StackLayer {
    #[serde(rename = "id")]
    #[builder(default)]
    pub id: LayerId,
    #[serde(rename = "nm")]
    #[builder(default)]
    pub name: String,
    #[serde(rename = "l")]
    #[builder(default)]
    pub layers: Vec<LayerId>,
    #[serde(rename = "s", skip_serializing_if = "Option::is_none", default)]
    #[builder(default)]
    pub size: Option<Vector2<u32>>,
}

impl StackLayer {
    pub fn push_layer(&mut self, id: LayerId) {
        self.layers.push(id);
    }
    
    pub fn remove_layer(&mut self, id: LayerId) {
        if let Some(idx) = self.layers.iter().position(|x| x == &id) {
            self.layers.remove(idx);
        }
    }

    pub fn replace_layers(&mut self, layers: Vec<LayerId>) -> Vec<LayerId> {
        std::mem::replace(&mut self.layers, layers)
    }
}