use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;

use crate::resources::DefaultMeshType;

#[derive(Component)]
pub struct VisualComponent {
    pub color: Color,
    pub mesh_id: usize,
}

impl VisualComponent {
    pub fn new(mesh_id: usize, color: Color) -> Self {
        let mut clamped_mesh_id = mesh_id;
        if clamped_mesh_id > DefaultMeshType::RotationRing as usize {
            clamped_mesh_id = 0;
        }

        Self {
            color,
            mesh_id: clamped_mesh_id,
        }
    }
}
