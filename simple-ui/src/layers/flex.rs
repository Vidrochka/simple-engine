use derive_builder::Builder;
use serde::Serialize;

use crate::{layer_id::LayerId, style::FillStyle};


#[derive(Debug, Serialize, Builder, Clone)]
#[builder(try_setter, setter(into))]
pub struct FlexLayer {
    #[serde(rename = "id")]
    #[builder(default)]
    pub id: LayerId,
    #[serde(rename = "nm")]
    #[builder(default)]
    pub name: String,
    #[serde(rename = "d")]
    #[builder(default)]
    pub direction: FlexDirection,
    #[serde(rename = "g", default)]
    #[builder(default)]
    pub gap: u16,
    #[serde(rename = "l")]
    #[builder(default)]
    pub layers: Vec<LayerId>,

    #[serde(rename = "fill", skip_serializing_if = "Option::is_none", default)]
    #[builder(default)]
    pub fill: Option<FillStyle>,

    #[serde(rename = "wr")]
    #[builder(default)]
    pub wrap: bool,

    #[serde(rename = "pr")]
    #[builder(default)]
    pub per_row: u16,

    #[serde(rename = "jc")]
    #[builder(default)]
    pub justify_content: JustifyContent,
}

impl FlexLayer {
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

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    #[serde(rename = "v")]
    Vertical,
    #[serde(rename = "h")]
    #[default]
    Horizontal
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    #[serde(rename = "s")]
    #[default]
    Start,
    #[serde(rename = "e")]
    End,
    #[serde(rename = "sb")]
    SpaceBetween
}