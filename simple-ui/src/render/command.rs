use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use mint::{Vector2, Vector3};
use uuid::Uuid;

use crate::{
    layer_id::LayerId, layers::{FlexDirection, FlexLayer, Layer, ShapesLayer, StackLayer}, style::UIMaterial, view::View
};

pub struct UIViewRender {}

pub trait IUIWriter {
    /// Add new shape to writer
    fn write_shape(&mut self, layer_name: String, points: Vec<Vector3<f32>>, indexes: Vec<u16>, material_name: String);

    /// Add ui material and return material name (Suggestion: add material cache)
    fn add_material(&mut self, material: UIMaterial) -> String;
}

impl UIViewRender {
    pub fn new() -> Self {
        Self {}
    }

    pub fn write_view(&self, view: &View, writer: &mut impl IUIWriter) {
        let root_id = view.root_id();

        self.build_render_command_recursive(
            writer,
            root_id,
            view,
            Vector3::from([0.0, 0.0, 0.0]),
        );
    }

    pub fn build_render_command_recursive(
        &self,
        writer: &mut impl IUIWriter,
        layer_id: &LayerId,
        view: &View,
        offset: Vector3<f32>,
    ) -> Vector2<f32> {
        let layer = view.layer(layer_id).expect(&format!("Expected layer id: [{layer_id:?}]"));

        match layer {
            Layer::Flex(flex_layer) => {
                self.build_flex_commands(flex_layer, writer, view, offset)
            }
            Layer::Stack(stack_layer) => {
                self.build_stack_commands(stack_layer, writer, view, offset)
            }
            Layer::Shape(shape_layer) => {
                self.build_shape_commands(shape_layer, writer, offset, view)
            }
        }
    }

    fn build_flex_commands(
        &self,
        flex_layer: &FlexLayer,
        writer: &mut impl IUIWriter,
        view: &View,
        offset: Vector3<f32>,
    ) -> Vector2<f32> {
        let mut bounds = Vector2::from([0f32, 0f32]);

        for (idx, layer_id) in flex_layer.layers.iter().enumerate() {
            match flex_layer.justify_content {
                crate::layers::JustifyContent::Start => {
                    let layer_offset = match flex_layer.direction {
                        FlexDirection::Horizontal => Vector3::from([bounds.x + offset.x, offset.y, offset.z]),
                        FlexDirection::Vertical => Vector3::from([offset.x, bounds.y + offset.y, offset.z]),
                    };

                    let layer_bounds = self.build_render_command_recursive(
                        writer,
                        layer_id,
                        view,
                        layer_offset,
                    );

                    match flex_layer.direction {
                        FlexDirection::Horizontal => {
                            bounds.x += layer_bounds.x;
                            bounds.y = bounds.y.max(layer_bounds.y);
                            
                            if idx != flex_layer.layers.len() - 1 {
                                bounds.x += flex_layer.gap as f32 / view.size().x as f32;
                            }
                        }
                        FlexDirection::Vertical => {
                            bounds.y += layer_bounds.y;
                            bounds.x = bounds.x.max(layer_bounds.x);

                            if idx != flex_layer.layers.len() - 1 {
                                bounds.y += flex_layer.gap as f32  / view.size().y as f32;
                            }
                        }
                    }
                }
                crate::layers::JustifyContent::End => todo!(),
                crate::layers::JustifyContent::SpaceBetween => todo!(),
            }
        }

        if let Some(fill) = &flex_layer.fill {
            let material_name = writer.add_material(UIMaterial {
                color: fill.color.clone(),
            });

            let points = vec![
                Vector3::from([offset.x, offset.y, offset.z]),
                Vector3::from([offset.x + bounds.x, offset.y, offset.z]),
                Vector3::from([offset.x + bounds.x, offset.y + bounds.y, offset.z]),
                Vector3::from([offset.x, offset.y + bounds.y, offset.z]),
            ]
            .into_iter()
            .map(|point| Vector3::from([point.x * 2f32 - 1f32, 1f32 - point.y * 2f32, point.z]))
            .collect::<Vec<_>>();

            let flat_ponts: Vec<f32> = points.iter().flat_map(|p| vec![p.x, p.y]).collect();
            let indexes = earcutr::earcut(&flat_ponts, &[], 2).ok().map(|x| x.into_iter().map(|x| x as u16).collect()).unwrap();

            writer.write_shape(
                flex_layer.name.clone(),
                points,
                indexes,
                material_name
            );
        }

        bounds
    }

    fn build_shape_commands(
        &self,
        shape_layer: &ShapesLayer,
        writer: &mut impl IUIWriter,
        offset: Vector3<f32>,
        view: &View,
    ) -> Vector2<f32> {
        for shape in &shape_layer.shapes {
            let material_name = writer.add_material(UIMaterial {
                color: shape.get_color().clone(),
            });

            let points = shape.get_points(&offset, view.size())
            .into_iter()
            .map(|point| Vector3::from([point.x * 2f32 - 1f32, 1f32 - point.y * 2f32, point.z]))
            .collect::<Vec<_>>();

            let flat_ponts: Vec<f32> = points.iter().flat_map(|p| vec![p.x, p.y]).collect();
            let indexes = earcutr::earcut(&flat_ponts, &[], 2).ok().map(|x| x.into_iter().map(|x| x as u16).collect()).unwrap();

            writer.write_shape(
                shape_layer.name.clone(),
                points,
                indexes,
                material_name
            );
        }

        shape_layer.bounds(view.size())
    }

    fn build_stack_commands(
        &self,
        stack_layer: &StackLayer,
        writer: &mut impl IUIWriter,
        view: &View,
        offset: Vector3<f32>,
    ) -> Vector2<f32> {
        let mut bounds = Vector2::from([0f32, 0f32]);

        for layer_id in &stack_layer.layers {
            //TODO: add stack size as inner bounds
            let layer_bounds =
                self.build_render_command_recursive(writer, layer_id, view, offset);

            bounds.x = bounds.x.max(layer_bounds.x);
            bounds.y = bounds.y.max(layer_bounds.y);
        }

        bounds
    }
}

// #[derive(Debug)]
// pub enum UIRenderCommand {
//     DrawShape {
//         layer_name: String,
//         points: Vec<Vector3<f32>>,
//         color: Vector3<u8>,
//     },
// }

// impl UIRenderCommand {
//     /// Генерирует индексы треангулированной фигуры
//     pub fn triangulate(&self) -> Option<Vec<u16>> {
//         match self {
//             UIRenderCommand::DrawShape { points, .. } => {
//                 let flat_ponts: Vec<f32> = points.iter().flat_map(|p| vec![p.x, p.y]).collect();
//                 earcutr::earcut(&flat_ponts, &[], 2).ok().map(|x| x.into_iter().map(|x| x as u16).collect())
//             },
//         }
//     }

//     pub fn layer_name(&self) -> &str {
//         match self {
//             UIRenderCommand::DrawShape { layer_name, .. } => layer_name.as_str(),
//         }
//     }
// }

