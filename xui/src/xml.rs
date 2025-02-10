use ahash::{AHashMap, RandomState};
use quick_xml::{events::{BytesStart, Event}, Reader};

use crate::{css::UICSSSource, node::{UINodeId, UINodeKind}, style::{FlexDirection, MarginBuilder, UIStyleId, UIStyleRules, Unit}, tree::UITree};

pub struct UIXmlSource {
    template: String,
    style_files_data: Vec<String>,
    prefix: Option<String>,
}

impl UIXmlSource {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
            style_files_data: Default::default(),
            prefix: None,
        }
    }

    pub fn add_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn add_style(mut self, style: impl Into<String>) -> Self {
        self.style_files_data.push(style.into());
        self
    }

    pub fn build(&self) -> UITree {
        let prefix = if let Some(prefix) = &self.prefix {
            format!("{prefix}.")
        } else {
            "".to_string()
        };

        let mut tree = UITree::new();

        let mut reader = Reader::from_str(&self.template);

        let mut ids = Vec::new();

        let default_styles = include_str!("../class-styles.css");

        tracing::info!("Default styles file '{default_styles}'");

        let default_css_source = UICSSSource::new(default_styles);

        let default_styles = default_css_source.build_styles();
        
        tracing::info!("Default styles {default_styles:?}");

        default_styles.into_iter().for_each(|style| {
            tree.add_styles(style);
        });

        self.style_files_data.iter().for_each(|css_file_data| {
            let styles = UICSSSource::new(css_file_data).build_styles();

            styles.into_iter().for_each(|style| tree.add_styles(style));
        });

        loop {
            match reader.read_event() {
                Ok(Event::Empty(e)) => {
                    parse_node(e, &mut tree, &mut ids, &prefix);
                }
                Ok(Event::Start(e)) => {
                    parse_node(e, &mut tree, &mut ids, &prefix);
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

fn parse_node(e: BytesStart<'_>, tree: &mut UITree, ids: &mut Vec<(String, UINodeId)>, prefix: &str) {
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

    let kind = match name.as_str() {
        "div" => UINodeKind::Div,
        tag => UINodeKind::Unknown(tag.to_string()),
    };

    let raw_id = if let Some(parent_id) = ids.last() {
        let sibling_count = tree.get_child_node_ids(&parent_id.1).count();
        if sibling_count > 0 {
            format!("{parent_id}.{name}[{sibling_count}]", parent_id = parent_id.0)
        } else {
            format!("{parent_id}.{name}", parent_id = parent_id.0)
        }
    } else {
        name.clone()
    };

    let node_id: UINodeId = format!("{prefix}{raw_id}").into();

    let parent = ids.last().cloned();

    let classes = attributes.get("classes").map(|x| {
        let classes = std::str::from_utf8(x).unwrap();
        classes.split(" ").map(|x| x.trim().to_string()).collect()
    }).unwrap_or_default();

    tree.add_node(node_id.clone(), kind, parent.map(|x| x.1), classes);

    ids.push((raw_id, node_id));
}