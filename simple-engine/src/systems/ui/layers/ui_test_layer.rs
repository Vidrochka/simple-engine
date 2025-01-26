use mint::Vector2;
use simple_layers::layer::ILayer;
use simple_ui::{layers::{FlexDirection, FlexLayerBuilder, Layer, RectangleShapeBuilder, Shape, ShapesLayerBuilder}, render::command::UIViewRender, style::FillStyleBuilder, view::*};
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use crate::systems::{render::{MaterialSystem, RenderCommandBuffer, RenderCommandsManager, RenderState}, ui::UIWriter};


#[derive(Debug)]
pub struct UITestLayer {
    render_commands_manager: RenderCommandsManager,
    material_system: MaterialSystem,

    render_state: RenderState,
}

impl UITestLayer {
    pub fn new(sp: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self {
            render_commands_manager: sp.resolve()?,
            material_system: sp.resolve()?,
            render_state: sp.resolve()?,
        })
    }
}

impl ILayer for UITestLayer {
    fn on_update(&mut self, _dt: &chrono::TimeDelta, scheduler: &mut simple_layers::scheduler::LayerScheduler) {
        let render_state = self.render_state.clone();
        let material_system = self.material_system.clone();
        let render_commands_manager = self.render_commands_manager.clone();

        scheduler.schedule(async move {
            let mut render_command_buffer = RenderCommandBuffer::new();

            let mut ui_writer = UIWriter::new(&mut render_command_buffer, &material_system);
    
            let render_state_lock = render_state.get();
    
            let Some(render_state) = render_state_lock.as_ref() else {
                return;
            };
    
            build_commands(render_state.size, &mut ui_writer);
    
            render_commands_manager.add_buffer(render_command_buffer);
        }, ());
    }
}


fn build_commands(size: Vector2<u32>, ui_writer: &mut UIWriter<'_>) {
    let mut view = View::new("Test view", size);

    let mut partial_view = PartialView::new();

    partial_view.add_layer(Layer::Shape(ShapesLayerBuilder::default()
        .id("1")
        .name("Shape layer 1")
        .shapes(vec![
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([40, 40])
                    .position([10, 40])
                    .fill(FillStyleBuilder::default().color([20, 20, 20]).build().unwrap())
                    .build()
                    .unwrap()
            ),
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([60, 60])
                    .position([2, 12])
                    .fill(FillStyleBuilder::default().color([0, 255, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
        ])
        .build()
        .unwrap())
    );

    let id = partial_view.add_layer(Layer::Flex(FlexLayerBuilder::default()
        .id("2")
        .name("Flex layer 2")
        .direction(FlexDirection::Vertical)
        // .fill(FillStyleBuilder::default().color([0, 0, 0, 255]).build().unwrap())
        .gap(10u16)
        .build()
        .unwrap())
    );

    partial_view.add_child_layer(&id, Layer::Shape(ShapesLayerBuilder::default().id("3")
        .name("Shape layer 2")
        .shapes(vec![
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([40, 40])
                    .position([30, 2])
                    .fill(FillStyleBuilder::default().color([0, 255, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([70, 70])
                    .position([14, 2])
                    .fill(FillStyleBuilder::default().color([0, 0, 255]).build().unwrap())
                    .build()
                    .unwrap()
            ),
        ])
        .build()
        .unwrap()
    ));

    partial_view.add_child_layer(&id, Layer::Shape(ShapesLayerBuilder::default().id("4")
        .name("Shape layer 3")
        .shapes(vec![
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([200, 200])
                    .position([2, 2])
                    .fill(FillStyleBuilder::default().color([0, 255, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
            Shape::Rectangle(
                RectangleShapeBuilder::default()
                    .size([300, 300])
                    .position([14, 2])
                    .fill(FillStyleBuilder::default().color([255, 0, 0]).build().unwrap())
                    .build()
                    .unwrap()
            ),
        ])
        .build()
        .unwrap()
    ));

    let id = view.add_view_layer(Layer::Flex(FlexLayerBuilder::default()
        .id("5")
        .name("Flex layer")
        .gap(10u16)
        // .fill(FillStyleBuilder::default().color([0, 0, 255, 0]).build().unwrap())
        .build()
        .unwrap())
    );

    view.replace_child_layers(id, partial_view);

    let render = UIViewRender::new();

    render.write_view(&view, ui_writer);
}