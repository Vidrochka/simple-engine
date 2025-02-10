use std::sync::Arc;

use mint::Vector2;
use parking_lot::{Mutex, RwLock};
use simple_layers::layer::ILayer;
// use simple_ui::{layers::{FlexDirection, FlexLayerBuilder, Layer, RectangleShapeBuilder, Shape, ShapesLayerBuilder}, render::writer::UIViewRenderWriter, source::xml::XmlSource, style::FillStyleBuilder, tree::*, UIControlEvent, UIMouseButton};
use xdi::{types::error::ServiceBuildResult, ServiceProvider};
use xui::{component::ComponentsRegistry, tree::UITree, view::UIView, xml::UIXmlSource};

use crate::{
    systems::{
        render::{MaterialSystem, RenderCommandBuffer, RenderCommandsManager, RenderState},
        ui::UIWriter
    },
    ui::components::button::ButtonComponent,
};


#[derive(Debug)]
pub struct UITestLayer {
    render_commands_manager: RenderCommandsManager,
    material_system: MaterialSystem,

    render_state: RenderState,

    ui_view: Arc<Mutex<UIView>>,
}

impl UITestLayer {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        let component_registry = ComponentsRegistry::new();

        component_registry.register_component(ButtonComponent::default());

        let ui_tree = component_registry.build_components_tree("button");

        // let ui_source = UIXmlSource::new(r#"
        //     <div classes="w-30 h-20 bg-blue test-color">
        //         <div classes="w-20 bg-red">
        //         </div>
        //         <div classes="w-10 bg-green">
        //         </div>
        //     </div>
        // "#);

        // let ui_tree = ui_source.build();

        let ui_view = UIView::new(Vector2::from([1920.0, 1080.0]), Arc::new(RwLock::new(ui_tree)));

        Ok(Self {
            render_commands_manager: sp.resolve()?,
            material_system: sp.resolve()?,
            render_state: sp.resolve()?,
            ui_view: Arc::new(Mutex::new(ui_view))
        })
    }
}

impl ILayer for UITestLayer {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, scheduler: &mut simple_layers::scheduler::LayerScheduler) {
        let render_state = self.render_state.clone();
        let material_system = self.material_system.clone();
        let render_commands_manager = self.render_commands_manager.clone();
        let ui_view = self.ui_view.clone();

        scheduler.schedule(async move {
            let render_state_lock = render_state.get();
    
            let Some(render_state) = render_state_lock.as_ref() else {
                return;
            };

            let mut render_command_buffer = RenderCommandBuffer::new();

            let mut ui_writer = UIWriter::new(&mut render_command_buffer, &material_system, [render_state.size.x, render_state.size.y]);
    
            // let mut ui_target_writer = UIEventTargetWriter::new();

            let mut ui_view = ui_view.lock();

            ui_view.resize([render_state.size.x as f32, render_state.size.y as f32].into());

            ui_view.build_draw_commands(&mut ui_writer);
    
            // build_commands(render_state.size, &mut ui_writer, &[UIControlEvent::MouseButtonDown { btn: UIMouseButton::Rignt, position: [20, 20].into() }], &mut ui_target_writer);
    
            render_commands_manager.add_buffer(render_command_buffer);
        }, ());
    }
}


// fn build_commands(size: Vector2<u32>, ui_writer: &mut UIWriter<'_>, events: &[UIControlEvent], ui_event_target_writer: &mut UIEventTargetWriter) {
//     let partial_view = XmlSource::parse(r##"
//         <shape name="left shape" >
//             <rect size="[60, 60]" position="[2, 12]" color="#00FF00"/>
//             <rect size="[40, 40]" position="[10, 40]" color="#141414"/>
//         </shape>
//         <flex name="Right flex" direction="column" gap="10">
//             <shape name="right top shape">
//                 <rect size="[70, 70]" position="[14, 2]" color="rgb(0, 0, 255)"/>
//                 <rect size="[40, 40]" position="[30, 2]" color="rgb(0, 255, 0)"/>
//             </shape>
//             <shape name="right bottom shape">
//                 <rect size="[300, 300]" position="[14, 2]" color="rgb(255, 0, 0)"/>
//                 <rect size="[200, 200]" position="[2, 2]" color="rgb(0, 255, 0)"/>
//             </shape>
//         </flex>
//     "##);
    
//     let mut view = UITree::new("Test view");

//     let id = view.add_view_layer(Layer::Flex(FlexLayerBuilder::default()
//         .id("5")
//         .name("Base flex")
//         .gap(30u16)
//         // .fill(FillStyleBuilder::default().color([0, 0, 255, 0]).build().unwrap())
//         .build()
//         .unwrap())
//     );

//     view.replace_child_layers(id, partial_view);

//     UIViewRenderWriter::write_view(&view, ui_writer, events, ui_event_target_writer);
// }