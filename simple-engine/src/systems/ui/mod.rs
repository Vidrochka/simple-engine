pub mod layers;
pub use layers::*;

use mint::{Vector2, Vector3, Vector4};
use xui::{node::UINodeId, style::UIMaterial, view::IUIWriter};

use super::render::{Material, MaterialSystem, RenderCommand, RenderCommandBuffer};

pub struct UIWriter<'a> {
    render_command_buffer: &'a mut RenderCommandBuffer,
    material_system: &'a MaterialSystem,
    view_size: Vector2<f32>,
}

impl<'a> UIWriter<'a> {
    pub fn new(
        render_command_buffer: &'a mut RenderCommandBuffer,
        material_system: &'a MaterialSystem,
        view_size: impl Into<Vector2<u32>>,
    ) -> Self {
        let view_size = view_size.into();
        
        Self {
            render_command_buffer,
            material_system,
            view_size: [view_size.x as f32, view_size.y as f32].into(),
        }
    }
}

impl IUIWriter for UIWriter<'_> {
    fn write_shape(
        &mut self,
        id: &UINodeId,
        points: Vec<Vector3<f32>>,
        indexes: Vec<u16>,
        material_name: String,
    ) {
        // tracing::info!("{layer_name} {points:?}");

        let points: Vec<_> = points
            .into_iter()
            .map(|point| {
                Vector3::from([
                    ((point.x as f32 / self.view_size.x * 2f32) - 1f32),
                    (1f32 - (point.y as f32 / self.view_size.y * 2 as f32)),
                    point.z as f32,
                ])
            })
            .collect();

        let indexes = indexes.into_iter().array_chunks::<3>().flat_map(|t| vec![t[0], t[2], t[1]])
            .collect::<Vec<_>>();

        // tracing::info!("{layer_name} {points:?}");

        self.render_command_buffer
            .push(RenderCommand::indexed_shape(material_name, points, indexes));
    }

    fn add_material(&mut self, material: UIMaterial) -> String {
        self.material_system.add_material(Material {
            color: [material.color.x, material.color.y, material.color.z, 0].into(),
        })
    }
}

// pub struct UIEventTargetWriter {}

// impl UIEventTargetWriter {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl IUIEventTargetWriter for UIEventTargetWriter {
//     fn target<'a>(
//         &mut self,
//         layer_id: &simple_ui::layer_id::LayerId,
//         event: &'a simple_ui::UIControlEvent,
//     ) {
//         // tracing::error!("Targeting layer {:?} with event {:?}", layer_id, event);
//     }
// }
