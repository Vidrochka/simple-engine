use ahash::{AHashMap, AHashSet, HashMap};
use itertools::Itertools;
use petgraph::{graph::{DiGraph, NodeIndex}, visit::{EdgeRef, IntoNodeIdentifiers}, Direction};

use crate::{node::{UINode, UINodeId, UINodeKind}, style::{StyleClass, UIStyle, UIStyleId, UIStyleRules}};

#[derive(Debug, Default)]
pub struct UITree {
    nodes_tree: DiGraph<UINode, usize>,

    id_to_tree_idx: HashMap<UINodeId, NodeIndex>,

    // nodes: AHashMap<UINodeId, UINode>,
    classes: AHashMap<String, StyleClass>,

    nodes_with_deprecated_styles: AHashSet<UINodeId>,
}

impl UITree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn root_ids(&self) -> impl Iterator<Item = &UINodeId> {
        self.nodes_tree.externals(Direction::Incoming).map(|x| self.nodes_tree.node_weight(x).unwrap().id())
    }

    pub fn leaf_ids(&self) -> impl Iterator<Item = &UINodeId> {
        self.nodes_tree.externals(Direction::Outgoing).map(|x| self.nodes_tree.node_weight(x).unwrap().id())
    }

    pub fn get_child_node_ids(&self, id: &UINodeId) -> impl Iterator<Item = &UINodeId> {
        self.nodes_tree.edges_directed(*self.id_to_tree_idx.get(id).unwrap(), Direction::Outgoing)
            .sorted_by_key(|x| x.weight())
            .map(|x| self.nodes_tree.node_weight(x.target()).unwrap().id())
    }

    pub fn get_parent(&self, id: &UINodeId) -> Option<&UINodeId> {
        self.nodes_tree.edges_directed(*self.id_to_tree_idx.get(id).unwrap(), Direction::Incoming)
            .next()
            .map(|x| self.nodes_tree.node_weight(x.source()).unwrap().id())
    }

    pub fn is_leaf(&self, id: &UINodeId) -> bool {
        self.nodes_tree.edges_directed(*self.id_to_tree_idx.get(id).unwrap(), Direction::Outgoing).count() == 0
    }

    pub fn ids(&self) -> impl Iterator<Item = &UINodeId> {
        self.nodes_tree.node_identifiers().map(|x| self.nodes_tree.node_weight(x).unwrap().id())
    }

    pub (crate) fn get_node(&self, id: &UINodeId) -> Option<&UINode> {
        self.id_to_tree_idx.get(id).and_then(|x| self.nodes_tree.node_weight(*x))
    }

    pub (crate) fn toposort(&self) -> impl DoubleEndedIterator<Item = &UINodeId> {
        crate::algo::stable_toposort(&self.nodes_tree).into_iter().map(|x| self.nodes_tree.node_weight(x).unwrap().id())
    }

    pub (crate) fn get_unknown_nodes(&self) -> impl Iterator<Item = &UINode> {
        self.nodes_tree.node_weights().filter(|x| x.is_unknown_kind())
    }

    pub fn add_node(&mut self, id: UINodeId, kind: UINodeKind, parent_id: Option<UINodeId>, classes: Vec<String>) {
        // assert!(!self.nodes_tree.contains_key(&id), "{}", id.to_string());
        // assert!(parent_id.as_ref().is_none_or(|x| self.nodes_tree.contains_key(&x)));

        tracing::info!("Add node id '{id:?}'");

        for class in &classes {
            self.classes.entry(class.clone()).or_default().add_node(id.clone());
        }

        let new_node_idx = self.nodes_tree.add_node(UINode::new(id.clone(), kind, classes, parent_id.clone()));

        if let Some(parent_id) = &parent_id {
            let parent_node_idx = self.id_to_tree_idx.get(parent_id).unwrap();

            let outgoing_edges_count = self.nodes_tree.edges_directed(*parent_node_idx, Direction::Outgoing).count();

            self.nodes_tree.add_edge(*parent_node_idx, new_node_idx, outgoing_edges_count);
        }

        self.id_to_tree_idx.insert(id.clone(), new_node_idx);

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
            let Some(node) = self.nodes_tree.node_weight_mut(*self.id_to_tree_idx.get(&node_id).unwrap()) else {
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