use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::style::UIStyleRules;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UINode {
    id: UINodeId,

    classes: Vec<String>,

    style: UIStyleRules,

    children: Vec<UINodeId>,
    parent: Option<UINodeId>,
}

impl UINode {
    pub fn new(id: UINodeId, classes: Vec<String>, parent: Option<UINodeId>) -> Self {
        Self {
            id,
            classes,
            style: Default::default(),
            children: Default::default(),
            parent,
        }
    }

    pub fn id(&self) -> &UINodeId {
        &self.id
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn children(&self) -> &[UINodeId] {
        &self.children
    }

    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    pub fn style(&self) -> &UIStyleRules {
        &self.style
    }

    pub fn set_style(&mut self, style: UIStyleRules) {
        self.style = style;
    }

    pub fn parent(&self) -> Option<&UINodeId> {
        self.parent.as_ref()
    }

    pub fn add_child(&mut self, id: UINodeId) {
        self.children.push(id);
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
        UINodeId(value)
    }
}

impl From<&str> for UINodeId {
    fn from(value: &str) -> Self {
        UINodeId(value.to_string())
    }
}

impl ToString for UINodeId {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}