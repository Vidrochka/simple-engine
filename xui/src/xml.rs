use ahash::AHashMap;
use quick_xml::{events::{BytesStart, Event}, Reader};

use crate::{css::UICSSSource, node::UINodeId, style::{FlexDirection, MarginBuilder, UIStyleId, UIStyleRules, Unit}, tree::UITree};



pub struct UIXmlSource {
    data: String
}

impl UIXmlSource {
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            data: data.into()
        }
    }

    pub fn build(&self) -> UITree {
        let mut tree = UITree::new();

        let mut reader = Reader::from_str(&self.data);

        let mut ids = Vec::new();

        let default_styles = include_str!("../class-styles.css");

        tracing::info!("Default styles file '{default_styles}'");

        let css_source = UICSSSource::new(default_styles);

        let styles = css_source.build_styles();
        
        tracing::info!("Default styles {styles:?}");

        styles.into_iter().for_each(|style| {
            tree.add_styles(style);
        });

        loop {
            match reader.read_event() {
                Ok(Event::Empty(e)) => {
                    parse_node(e, &mut tree, &mut ids);
                }
                Ok(Event::Start(e)) => {
                    parse_node(e, &mut tree, &mut ids);
                },
                Ok(Event::End(_)) => {
                    ids.pop();
                },
                Ok(Event::Eof) => break,
                Ok(_) => {},
                Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            }
        }

        tree
    }
}

fn parse_node(e: BytesStart<'_>, tree: &mut UITree, ids: &mut Vec<UINodeId>) {
    let name = e.name();

    let attributes = e.attributes()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .into_iter()
        .map(|x| {
            let key = std::str::from_utf8(x.key.0).unwrap().to_string();
            (key, x.value)
        })
        .collect::<AHashMap<_, _>>();

    let name = std::str::from_utf8(name.as_ref()).unwrap().to_string();

    if name != "div" {
        panic!("Unknown node type: {}", name)
    }

    let id = if let Some(parent_id) = ids.last() {
        let sibling_count = tree.get_node(parent_id).unwrap().children().len();
        if sibling_count > 0 {
            format!("{parent_id}.{name}[{sibling_count}]", parent_id = parent_id.to_string())
        } else {
            format!("{parent_id}.{name}", parent_id = parent_id.to_string())
        }
    } else {
        name.clone()
    };

    let node_id: UINodeId = id.into();

    let parent = ids.last().cloned();

    let classes = attributes.get("classes").map(|x| {
        let classes = std::str::from_utf8(x).unwrap();
        classes.split(" ").map(|x| x.trim().to_string()).collect()
    }).unwrap_or_default();

    tree.add_node(node_id.clone(), parent, classes);

    ids.push(node_id);
}