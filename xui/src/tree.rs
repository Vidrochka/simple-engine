use ahash::{AHashMap, AHashSet};

use crate::{node::{UINode, UINodeId}, style::{StyleClass, UIStyle, UIStyleId, UIStyleRules}};

#[derive(Debug, Default)]
pub struct UITree {
    nodes: AHashMap<UINodeId, UINode>,
    classes: AHashMap<String, StyleClass>,

    nodes_with_deprecated_styles: AHashSet<UINodeId>,
}

impl UITree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn root_ids(&self) -> impl Iterator<Item = &UINodeId> {
        self.nodes.iter().filter_map(|(k, v)| {
            if v.is_root() {
                Some(k)
            } else {
                None
            }
        })
    }

    pub fn ids(&self) -> impl Iterator<Item = &UINodeId> {
        self.nodes.keys()
    }

    pub (crate) fn get_node(&self, id: &UINodeId) -> Option<&UINode> {
        self.nodes.get(id)
    }

    pub fn add_node(&mut self, id: UINodeId, parent_id: Option<UINodeId>, classes: Vec<String>) {
        assert!(!self.nodes.contains_key(&id), "{}", id.to_string());
        assert!(parent_id.as_ref().is_none_or(|x| self.nodes.contains_key(&x)));

        for class in &classes {
            self.classes.entry(class.clone()).or_default().add_node(id.clone());
        }

        if let Some(parent_id) = &parent_id {
            self.nodes.get_mut(parent_id).unwrap().add_child(id.clone());
        }

        self.nodes.insert(id.clone(), UINode::new(id.clone(), classes, parent_id));

        self.nodes_with_deprecated_styles.insert(id);
    }

    pub fn add_styles(&mut self, style: UIStyle) {
        tracing::info!("Add style {style:?}");

        let class = self.classes.entry(style.class.clone()).or_default();

        class.add_styles(style.id, style.rules);

        self.nodes_with_deprecated_styles.extend(class.node_ids().iter().cloned().collect::<Vec<_>>());
    }

    pub fn need_style_recalculation(&self) -> bool {
        !self.nodes_with_deprecated_styles.is_empty()
    }

    pub fn recalculate_styles(&mut self) {
        for node_id in self.nodes_with_deprecated_styles.drain() {
            let Some(node) = self.nodes.get_mut(&node_id) else {
                continue;
            };

            tracing::info!("Start node {node_id:?} style recalculation, classes {:?}", node.classes());

            let styles = node.classes().iter().filter_map(|class| self.classes.get(class).map(|x| x.styles())).flatten();

            let style = UIStyleRules::merge(styles);

            tracing::info!("Node {node_id:?} style recalculated {style:?}");

            node.set_style(style);
        }
    }
}