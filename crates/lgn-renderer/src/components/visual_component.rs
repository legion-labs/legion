use lgn_ecs::prelude::*;
use lgn_graphics_data::{runtime_mesh::MeshReferenceType, Color};

use crate::resources::DefaultMeshType;

#[derive(Component)]
pub struct VisualComponent {
    pub color: Color,
    pub mesh_id: usize,
    pub mesh_reference_type: Option<MeshReferenceType>,
}

impl VisualComponent {
    pub fn new(mesh_reference_type: Option<MeshReferenceType>, color: Color) -> Self {
        Self {
            color: Color::from((0, 255, 0)),
            mesh_id: 0,
            mesh_reference_type,
        }
    }

    pub fn new_default_mesh(mesh_type: DefaultMeshType, color: Color) -> Self {
        Self {
            color,
            mesh_id: mesh_type as usize,
            mesh_reference_type: None,
        }
    }
}
