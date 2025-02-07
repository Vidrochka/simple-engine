use mint::{Vector2, Vector3};

use crate::{layers::{FlexDirection, FlexLayerBuilder, Layer, RectangleShapeBuilder, Shape, ShapesLayerBuilder}, render::writer::{IUIEventTargetWriter, IUIWriter, UIViewRenderWriter}, style::{FillStyleBuilder, UIMaterial}, tree::{PartialUITree, UITree}};


#[test]
pub fn build_render_commands_ok() {
    let mut view = UITree::new("Test view");

    let mut partial_view = PartialUITree::new();

    partial_view.add_layer(Layer::Shape(ShapesLayerBuilder::default()
        .id("1")
        .name("Shape layer 1")
        .shapes(vec![
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([10, 10])
                    .position([2, 2])
                    .fill(FillStyleBuilder::default().color([255, 0, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([10, 10])
                    .position([14, 2])
                    .fill(FillStyleBuilder::default().color([0, 255, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
        ])
        .build()
        .unwrap())
    );

    let id = partial_view.add_layer(Layer::Flex(FlexLayerBuilder::default()
        .id("2")
        .name("Flex layer 2")
        .direction(FlexDirection::Vertical)
        .gap(10u16)
        .build()
        .unwrap())
    );

    partial_view.add_child_layer(&id, Layer::Shape(ShapesLayerBuilder::default().id("3")
        .name("Shape layer 2")
        .shapes(vec![
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([10, 10])
                    .position([2, 2])
                    .fill(FillStyleBuilder::default().color([30, 30, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([10, 10])
                    .position([14, 2])
                    .fill(FillStyleBuilder::default().color([0, 30, 30]).build().unwrap())
                    .build()
                    .unwrap()
            ),
        ])
        .build()
        .unwrap()
    ));

    partial_view.add_child_layer(&id, Layer::Shape(ShapesLayerBuilder::default().id("4")
        .name("Shape layer 3")
        .shapes(vec![
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([10, 10])
                    .position([2, 2])
                    .fill(FillStyleBuilder::default().color([150, 0, 150]).build().unwrap())
                    .build()
                    .unwrap()
            ),
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([10, 10])
                    .position([14, 2])
                    .fill(FillStyleBuilder::default().color([150, 150, 150]).build().unwrap())
                    .build()
                    .unwrap()
            ),
        ])
        .build()
        .unwrap()
    ));

    let id = view.add_view_layer(Layer::Flex(FlexLayerBuilder::default()
        .id("5")
        .name("Flex layer")
        .build()
        .unwrap())
    );

    view.replace_child_layers(id, partial_view);

    pub struct UIWriter;

    impl IUIWriter for UIWriter {
        fn write_shape(&mut self, layer_name: String, points: Vec<Vector3<u32>>, indexes: Vec<u16>, material_name: String) {
        }

        fn add_material(&mut self, material: UIMaterial) -> String {
            "".to_string()
        }
    }

    pub struct UIEventTargetWriter;

    impl IUIEventTargetWriter for UIEventTargetWriter {
        fn target<'a>(&mut self, layer_id: &crate::layer_id::LayerId, event: &'a crate::UIControlEvent) {
            
        }
    }

    UIViewRenderWriter::write_view(&view, &mut UIWriter, &vec![], &mut UIEventTargetWriter);
} 