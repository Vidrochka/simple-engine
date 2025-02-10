use std::sync::Arc;

use parking_lot::RwLock;
use xui::{component::IComponentCallback, template, tree::UITree};

#[derive(Debug, Default)]
#[template(name = "button", template = "./button.xml", styles = "./button.css")]
pub struct ButtonComponent {
    text: String,
}

impl IComponentCallback for ButtonComponent {
    fn callback(&self, name: &str, args: &serde_json::Value, tree: &Arc<RwLock<UITree>>) {
        
    }
}

