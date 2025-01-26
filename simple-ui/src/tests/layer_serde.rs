use crate::{style::FillStyleBuilder, layers::{Layer, FlexLayerBuilder, RectangleShapeBuilder, Shape, ShapesLayerBuilder}};



#[test]
fn shape_rectangle_serde_ok() {
    let shape_layer = Layer::Shape(
        ShapesLayerBuilder::default()
            .id("0")
            .name("Test layer")
            .shapes(vec![
                Shape::Rectangle(
                    RectangleShapeBuilder::default()
                        .size([10, 10])
                        .position([2, 2])
                        .fill(FillStyleBuilder::default().color([255, 0, 0]).build().unwrap())
                        .build()
                        .unwrap()
                )
            ])
            .build()
            .unwrap()
    );
    
    let shape_layer_str = serde_json::to_string_pretty(&shape_layer).unwrap();

    // println!("{shape_layer_str}");

    assert_eq!(shape_layer_str, r#"{
  "ty": "sh",
  "id": "0",
  "nm": "Test layer",
  "sh": [
    {
      "ty": "rc",
      "p": [
        2,
        2
      ],
      "s": [
        10,
        10
      ],
      "fill": {
        "c": [
          255,
          0,
          0,
          255
        ]
      }
    }
  ]
}"#);
}


#[test]
fn shape_flex_serde_ok() {
    let flex_layer = Layer::Flex(FlexLayerBuilder::default()
        .id("0")
        .name("Test flex layer")
        .gap(2u16)
        .layers(vec![
            "1".into(),
            "2".into(),
        ])
        .build()
        .unwrap()
    );

    let flex_layer_str = serde_json::to_string_pretty(&flex_layer).unwrap();

    // println!("{flex_layer_str}");

    assert_eq!(flex_layer_str, r#"{
  "ty": "flex",
  "id": "0",
  "nm": "Test flex layer",
  "d": "h",
  "g": 2,
  "l": [
    "1",
    "2"
  ],
  "wr": false,
  "pr": 0
}"#);
}