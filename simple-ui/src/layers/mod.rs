use serde::Serialize;

pub mod stack;
pub use stack::*;

pub mod flex;
pub use flex::*;

pub mod shape;
pub use shape::*;

use crate::layer_id::LayerId;


#[derive(Debug, Serialize)]
#[serde(tag = "ty")]
pub enum Layer {
    /// Набор форм расположенных в 1 месте
    #[serde(rename = "sh")]
    Shape(ShapesLayer),
    /// Гибкий список элементов
    #[serde(rename = "flex")]
    Flex(FlexLayer),
    /// Слои перекрывающие друг друга стеком
    #[serde(rename = "stack")]
    Stack(StackLayer),
    // TODO: add layers
    // - Image
    // - Text
}

impl Layer {
    pub fn id(&self) -> &LayerId {
        match self {
            Layer::Shape(shapes_layer) => &shapes_layer.id,
            Layer::Flex(flex_layer) => &flex_layer.id,
            Layer::Stack(stack_layer) => &stack_layer.id,
        }
    }

    pub fn as_stack_mut(&mut self) -> Option<&mut StackLayer> {
        match self {
            Self::Stack(stack) => Some(stack),
            _ => None,
        }
    }
}