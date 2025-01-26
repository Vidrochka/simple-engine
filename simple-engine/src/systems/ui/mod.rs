pub mod layers;
pub use layers::*;

use mint::{Vector3, Vector4};
use simple_ui::{render::command::IUIWriter, style::UIMaterial};

use super::render::{Material, MaterialSystem, RenderCommand, RenderCommandBuffer};

pub struct UIWriter<'a> {
    render_command_buffer: &'a mut RenderCommandBuffer,
    material_system: &'a MaterialSystem,
}

impl<'a> UIWriter<'a> {
    pub fn new(
        render_command_buffer: &'a mut RenderCommandBuffer,
        material_system: &'a MaterialSystem,
    ) -> Self {
        Self {
            render_command_buffer,
            material_system,
        }
    }
}

impl IUIWriter for UIWriter<'_> {
    fn write_shape(
        &mut self,
        _layer_name: String,
        points: Vec<Vector3<f32>>,
        indexes: Vec<u16>,
        material_name: String,
    ) {
        self.render_command_buffer
            .push(RenderCommand::indexed_shape(
                material_name,
                points,
                indexes,
            ));
    }

    fn add_material(&mut self, material: UIMaterial) -> String {
        self.material_system.add_material(Material {
            color: [material.color.x, material.color.y, material.color.z, 0].into(),
        })
    }
}
