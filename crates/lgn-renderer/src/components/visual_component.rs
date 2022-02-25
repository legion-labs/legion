use std::str::FromStr;

use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use lgn_ecs::prelude::*;
use lgn_graphics_data::{runtime::ModelReferenceType, Color};

use crate::resources::{DefaultMeshType, DEFAULT_MESH_GUIDS};

#[derive(Component)]
pub struct VisualComponent {
    pub color: Color,
    pub model_reference: Option<ResourceTypeAndId>,
}

impl VisualComponent {
    pub fn new(model_reference: &Option<ModelReferenceType>, color: Color) -> Self {
        Self {
            color,
            model_reference: model_reference.as_ref().map(ModelReferenceType::id),
        }
    }

    pub fn new_default_mesh(mesh_type: DefaultMeshType, color: Color) -> Self {
        Self {
            color,
            model_reference: Some(ResourceTypeAndId {
                kind: ResourceType::from_raw(1),
                id: ResourceId::from_str(DEFAULT_MESH_GUIDS[mesh_type as usize]).unwrap(),
            }),
        }
    }
}
