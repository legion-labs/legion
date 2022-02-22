use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::*;
use lgn_graphics_data::{runtime::ModelReferenceType, Color};

use crate::resources::DefaultMeshType;

#[derive(Component)]
pub struct VisualComponent {
    pub color: Color,
    pub mesh_id: usize,
    pub model_reference: Option<ResourceTypeAndId>,
}

impl VisualComponent {
    pub fn new(model_reference: &Option<ModelReferenceType>, color: Color) -> Self {
        Self {
            color,
            mesh_id: 0,
            model_reference: model_reference.as_ref().map(ModelReferenceType::id),
        }
    }

    pub fn new_default_mesh(mesh_type: DefaultMeshType, color: Color) -> Self {
        Self {
            color,
            mesh_id: mesh_type as usize,
            model_reference: None,
        }
    }
}
