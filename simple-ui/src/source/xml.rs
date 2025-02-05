use std::collections::HashMap;

use mint::Vector2;
use quick_xml::{events::{BytesStart, Event}, Reader};

use crate::{layer_id::LayerId, layers::*, style::FillStyleBuilder, tree::PartialUITree};



pub struct XmlSource {}

impl XmlSource {
    pub fn parse(xml: &str) -> PartialUITree {
        let mut view = PartialUITree::new();

        let mut reader = Reader::from_str(xml);

        let mut ids = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Empty(e)) => {
                    parse_element(e, &mut view, &mut ids);
                }
                Ok(Event::Start(e)) => {
                    parse_element(e, &mut view, &mut ids);
                },
                Ok(Event::End(_)) => {
                    ids.pop();
                },
                Ok(Event::Eof) => break,
                Ok(_) => {},
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            }
        }

        view
    }
}

fn parse_element(e: BytesStart<'_>, view: &mut PartialUITree, ids: &mut Vec<LayerId>) {
    let name = e.name();
    let attributes = e.attributes()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .into_iter()
        .map(|x| {
            let key = std::str::from_utf8(x.key.0).unwrap().to_string();
            (key, x.value)
        })
        .collect::<HashMap<_, _, ahash::RandomState>>();

    let layer = match name.as_ref() {
        b"shape" => {
            let mut builder = ShapesLayerBuilder::default();

            if let Some(id) = attributes.get("id") {
                builder.id(std::str::from_utf8(id).unwrap());
            }

            if let Some(name) = attributes.get("name") {
                builder.name(std::str::from_utf8(name).unwrap());
            }

            Layer::Shape(builder.build().unwrap())
        },
        b"rect" => {
            let mut builder = RectangleShapeBuilder::default();

            let size = attributes.get("size").unwrap();

            let size = serde_json::from_str::<Vector2<u32>>(std::str::from_utf8(size).unwrap()).unwrap();

            builder.size(size);

            if let Some(position) = attributes.get("position") {
                let position = serde_json::from_str::<Vector2<u32>>(std::str::from_utf8(position).unwrap()).unwrap();
                
                builder.position(position);
            } else {
                builder.position([0, 0]);
            }

            if let Some(wrap) = attributes.get("color") {
                let color = std::str::from_utf8(wrap).unwrap();
                let color = csscolorparser::parse(color).unwrap().to_rgba8();

                builder.fill(FillStyleBuilder::default().color([
                    color[0],
                    color[1],
                    color[2]
                ]).build().unwrap());
            }

            let shape = Shape::Rectangle(builder.build().unwrap());

            let id = ids.last().unwrap();

            view.get_layer_mut(id).unwrap().as_shape_mut().unwrap().shapes.push(shape);
            
            return;
        },
        b"flex" => {
            let mut builder = FlexLayerBuilder::default();

            if let Some(id) = attributes.get("id") {
                builder.id(std::str::from_utf8(id).unwrap());
            }

            if let Some(name) = attributes.get("name") {
                builder.name(std::str::from_utf8(name).unwrap());
            }

            if let Some(direction) = attributes.get("direction") {
                match std::str::from_utf8(direction).unwrap() {
                    "row" => builder.direction(FlexDirection::Horizontal),
                    "column" => builder.direction(FlexDirection::Vertical),
                    _ => panic!("Unknown flex direction: {:?}", direction),
                };
            }

            if let Some(direction) = attributes.get("justify-content") {
                match std::str::from_utf8(direction).unwrap() {
                    "start" => builder.justify_content(JustifyContent::Start),
                    "end" => builder.justify_content(JustifyContent::Start),
                    "space-between" => builder.justify_content(JustifyContent::SpaceBetween),
                    _ => panic!("Unknown justify contern: {:?}", direction),
                };
            }


            if let Some(gap) = attributes.get("gap") {
                let gap = std::str::from_utf8(gap).unwrap().parse::<u16>().unwrap();
                builder.gap(gap);
            }

            if let Some(wrap) = attributes.get("wrap") {
                let wrap = std::str::from_utf8(wrap).unwrap().parse::<bool>().unwrap();
                builder.wrap(wrap);
            }

            if let Some(wrap) = attributes.get("color") {
                let color = csscolorparser::parse(std::str::from_utf8(wrap).unwrap()).unwrap().to_rgba8();

                builder.fill(FillStyleBuilder::default().color([
                    color[0],
                    color[1],
                    color[2]
                ]).build().unwrap());
            }

            Layer::Flex(builder.build().unwrap())
        },
        b"stack" => {
            let mut builder = StackLayerBuilder::default();

            if let Some(id) = attributes.get("id") {
                builder.id(std::str::from_utf8(id).unwrap());
            }

            if let Some(name) = attributes.get("name") {
                builder.name(std::str::from_utf8(name).unwrap());
            }

            Layer::Stack(builder.build().unwrap())
        },
        _ => panic!("Unknown layer type: {:?}", name),
    };

    if let Some(parent_id) = ids.last() {
        let id = view.add_child_layer(parent_id, layer);
        ids.push(id);
    } else {
        let id = view.add_layer(layer);
        ids.push(id);
    }
}