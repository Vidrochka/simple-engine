use std::sync::Arc;

use ahash::AHashMap;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::{tree::UITree, xml::UIXmlSource};

pub struct ComponentsRegistry {
    component_templates: Arc<RwLock<AHashMap<String, ComponentFactory>>>,
}

impl ComponentsRegistry {
    pub fn new() -> Self {
        Self {
            component_templates: Default::default()
        }
    }

    pub fn register_component(&self, component_template_factory: impl IComponentTemplateFactory + 'static) {
        self.component_templates.write().insert(
            component_template_factory.name(),
            ComponentFactory::new(component_template_factory),
        );
    }

    pub fn build_components_tree(&self, main_component_name: &str) -> UITree {
        let components = self.component_templates.read();

        let component_factory = components.get(main_component_name).unwrap();

        // let mut components_tree = ComponentsTree::new();

        let component_template = component_factory.build_template();

        let component_id = Uuid::new_v4();

        let ui_tree_source = UIXmlSource::new(component_template)
            .add_prefix(component_id);

        let ui_tree_source = component_factory.build_styles().into_iter().fold(ui_tree_source, |ui_tree_source, style| {
            ui_tree_source.add_style(style)
        });

        ui_tree_source.build()

        // ui_tree.get_unknown_nodes();
    }
}

pub struct ComponentFactory {
    component_template: Box<dyn IComponentTemplateFactory>,
}

impl ComponentFactory {
    pub fn new(component_template: impl IComponentTemplateFactory + 'static) -> Self {
        Self { component_template: Box::new(component_template) }
    }

    pub fn build_template(&self) -> String {
        self.component_template.template()
    }

    pub fn build_styles(&self) -> Vec<String> {
        self.component_template.styles()
    }
}

pub struct ComponentsTree {
    components: AHashMap<ComponentId, Component>,
}

impl ComponentsTree {
    pub fn new() -> Self {
        Self {
            components: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ComponentId(Uuid);

impl ComponentId {
    fn new() -> ComponentId {
        ComponentId(Uuid::new_v4())
    }
}

pub struct Component {
    name: String,
    child_components: Vec<ComponentId>,
}

pub trait IComponentTemplateFactory: IComponentTemplate + IComponentCallback {}

pub trait IComponentTemplate {
    fn name(&self) -> String;
    fn template(&self) -> String;
    fn styles(&self) -> Vec<String>;
}

pub trait IComponentCallback {
    fn callback(&self, name: &str, args: &serde_json::Value, tree: &Arc<RwLock<UITree>>);
}

impl<T: IComponentCallback + IComponentTemplate> IComponentTemplateFactory for T {}