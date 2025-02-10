use ahash::RandomState;
use cfg_if::cfg_if;
use petgraph::csr::IndexType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::style::UIStyleRules;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UINode {
    id: UINodeId,

    kind: UINodeKind,

    classes: Vec<String>,

    style: UIStyleRules,

    // children: Vec<UINodeId>,
    // parent: Option<UINodeId>,
}

impl UINode {
    pub fn new(id: UINodeId, kind: UINodeKind, classes: Vec<String>, parent: Option<UINodeId>) -> Self {
        Self {
            id,
            kind,
            classes,
            style: Default::default(),
            // children: Default::default(),
            // parent,
        }
    }

    pub fn id(&self) -> &UINodeId {
        &self.id
    }

    // pub fn is_root(&self) -> bool {
    //     self.parent.is_none()
    // }

    // pub fn is_leaf(&self) -> bool {
    //     self.children.is_empty()
    // }

    // pub fn children(&self) -> &[UINodeId] {
    //     &self.children
    // }

    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    pub fn style(&self) -> &UIStyleRules {
        &self.style
    }

    pub fn set_style(&mut self, style: UIStyleRules) {
        self.style = style;
    }

    // pub fn parent(&self) -> Option<&UINodeId> {
    //     self.parent.as_ref()
    // }

    // pub fn add_child(&mut self, id: UINodeId) {
    //     self.children.push(id);
    // }

    pub fn is_unknown_kind(&self) -> bool {
        matches!(self.kind, UINodeKind::Unknown(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct UINodeId(String);

impl UINodeId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl From<String> for UINodeId {
    fn from(value: String) -> Self {
        cfg_if!(
            if #[cfg(feature = "debug-ids")] {
                let id = UINodeId(value.to_string());
            } else {
                let hash_builder = RandomState::default();
                let id = hash_builder.hash_one(value);
                let id = UINodeId(id.to_string());
            }
        );

        id
    }
}

impl From<&str> for UINodeId {
    fn from(value: &str) -> Self {
        cfg_if::cfg_if!(
            if #[cfg(feature = "debug-ids")] {
                let id = UINodeId(value.to_string());
            } else {
                let hash_builder = RandomState::default();
                let id = hash_builder.hash_one(value);
                let id = UINodeId(id.to_string());
            }
        );

        id
    }
}

impl ToString for UINodeId {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UINodeKind {
    Div,
    Unknown(String)
}