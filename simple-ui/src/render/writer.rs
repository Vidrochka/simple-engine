use mint::{Vector2, Vector3};

use crate::{
    UIControlEvent,
    layer_id::LayerId,
    layers::{FlexDirection, FlexLayer, Layer, ShapesLayer, StackLayer},
    style::UIMaterial,
    tree::UITree,
};

pub struct UIViewRenderWriter {}

pub trait IUIWriter {
    /// Add new shape to writer
    fn write_shape(
        &mut self,
        layer_name: String,
        points: Vec<Vector3<u32>>,
        indexes: Vec<u16>,
        material_name: String,
    );

    /// Add ui material and return material name (Suggestion: add material cache)
    fn add_material(&mut self, material: UIMaterial) -> String;
}

pub trait IUIEventTargetWriter {
    /// Add target event to writer
    fn target<'a>(&mut self, layer_id: &LayerId, event: &'a UIControlEvent);
}

impl UIViewRenderWriter {
    pub fn write_view(
        view: &UITree,
        writer: &mut impl IUIWriter,
        events: &[UIControlEvent],
        event_target_writer: &mut impl IUIEventTargetWriter,
    ) {
        let root_id = view.root_id();

        Self::build_render_command_recursive(
            writer,
            root_id,
            view,
            Vector3::from([0, 0, 0]),
            events,
            event_target_writer,
        );
    }

    fn build_render_command_recursive(
        writer: &mut impl IUIWriter,
        layer_id: &LayerId,
        view: &UITree,
        offset: Vector3<u32>,
        events: &[UIControlEvent],
        event_target_writer: &mut impl IUIEventTargetWriter,
    ) -> Vector2<u32> {
        let layer = view
            .layer(layer_id)
            .expect(&format!("Expected layer id: [{layer_id:?}]"));

        match layer {
            Layer::Flex(flex_layer) => Self::build_flex_commands(
                flex_layer,
                writer,
                view,
                offset,
                events,
                event_target_writer,
            ),
            Layer::Stack(stack_layer) => Self::build_stack_commands(
                stack_layer,
                writer,
                view,
                offset,
                events,
                event_target_writer,
            ),
            Layer::Shape(shape_layer) => Self::build_shape_commands(
                shape_layer,
                writer,
                offset,
                events,
                event_target_writer,
            ),
        }
    }

    fn build_flex_commands(
        flex_layer: &FlexLayer,
        writer: &mut impl IUIWriter,
        view: &UITree,
        offset: Vector3<u32>,
        events: &[UIControlEvent],
        event_target_writer: &mut impl IUIEventTargetWriter,
    ) -> Vector2<u32> {
        let mut bounds = Vector2::from([0, 0]);

        for (idx, layer_id) in flex_layer.layers.iter().enumerate() {
            match flex_layer.justify_content {
                crate::layers::JustifyContent::Start => {
                    let layer_offset = match flex_layer.direction {
                        FlexDirection::Horizontal => {
                            Vector3::from([bounds.x + offset.x, offset.y, offset.z])
                        }
                        FlexDirection::Vertical => {
                            Vector3::from([offset.x, bounds.y + offset.y, offset.z])
                        }
                    };

                    // tracing::info!("{} layer offset [{layer_offset:?}] [{offset:?}]", flex_layer.name);

                    let layer_bounds = Self::build_render_command_recursive(
                        writer,
                        layer_id,
                        view,
                        layer_offset,
                        events,
                        event_target_writer,
                    );

                    // tracing::info!("{} layer add offset to bound [{layer_bounds:?}]", flex_layer.name);

                    match flex_layer.direction {
                        FlexDirection::Horizontal => {
                            bounds.x += layer_bounds.x;
                            bounds.y = bounds.y.max(layer_bounds.y);

                            if idx != flex_layer.layers.len() - 1 {
                                bounds.x += flex_layer.gap as u32;
                            }
                        }
                        FlexDirection::Vertical => {
                            bounds.y += layer_bounds.y;
                            bounds.x = bounds.x.max(layer_bounds.x);

                            if idx != flex_layer.layers.len() - 1 {
                                bounds.y += flex_layer.gap as u32;
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
            ];

            let flat_ponts = points.iter().flat_map(|p| vec![p.x as f32, p.y as f32]).collect::<Vec<_>>();
            let indexes = earcutr::earcut(&flat_ponts, &[], 2)
                .ok()
                .map(|x| x.into_iter().map(|x| x as u16).collect())
                .unwrap();

            writer.write_shape(flex_layer.name.clone(), points, indexes, material_name);

            for event in events {
                if event.is_in_bound([offset.x, offset.y].into(), bounds) {
                    event_target_writer.target(&flex_layer.id, &event);
                }
            }
        }

        // tracing::info!("{} layer bound [{bounds:?}]", flex_layer.name);

        bounds
    }

    fn build_shape_commands(
        shape_layer: &ShapesLayer,
        writer: &mut impl IUIWriter,
        offset: Vector3<u32>,
        events: &[UIControlEvent],
        event_target_writer: &mut impl IUIEventTargetWriter,
    ) -> Vector2<u32> {
        let bounds = shape_layer.bounds();

        for shape in &shape_layer.shapes {
            let material_name = writer.add_material(UIMaterial {
                color: shape.get_color().clone(),
            });

            let points = shape
                .get_points(&offset);

            let flat_ponts = points.iter().flat_map(|p| vec![p.x as f32, p.y as f32]).collect::<Vec<_>>();
            let indexes = earcutr::earcut(&flat_ponts, &[], 2)
                .ok()
                .map(|x| x.into_iter().map(|x| x as u16).collect())
                .unwrap();

            writer.write_shape(shape_layer.name.clone(), points, indexes, material_name);
        }

        for event in events {
            if event.is_in_bound([offset.x, offset.y].into(), bounds) {
                event_target_writer.target(&shape_layer.id, &event);
            }

            // tracing::warn!("{event:?} {offset:?} {bounds:?}");
        }

        bounds
    }

    fn build_stack_commands(
        stack_layer: &StackLayer,
        writer: &mut impl IUIWriter,
        view: &UITree,
        offset: Vector3<u32>,
        events: &[UIControlEvent],
        event_target_writer: &mut impl IUIEventTargetWriter,
    ) -> Vector2<u32> {
        let mut bounds = Vector2::from([0, 0]);

        for layer_id in &stack_layer.layers {
            //TODO: add stack size as inner bounds
            let layer_bounds = Self::build_render_command_recursive(
                writer,
                layer_id,
                view,
                offset,
                events,
                event_target_writer,
            );

            bounds.x = bounds.x.max(layer_bounds.x);
            bounds.y = bounds.y.max(layer_bounds.y);
        }

        for event in events {
            if event.is_in_bound([offset.x, offset.y].into(), bounds) {
                event_target_writer.target(&stack_layer.id, &event);
            }
        }

        bounds
    }
}
