use std::str::FromStr;

use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use lgn_ecs::prelude::*;
use lgn_graphics_data::{runtime::ModelReferenceType, Color};

use crate::resources::{DefaultMeshType, DEFAULT_MESH_GUIDS};

#[derive(Component)]
pub struct VisualComponent {
    pub color: Color,
    pub color_blend: f32,
    pub model_resource_id: Option<ResourceTypeAndId>,
}

impl VisualComponent {
    pub fn new(
        model_resource_id: &Option<ModelReferenceType>,
        color: Color,
        color_blend: f32,
    ) -> Self {
        Self {
            color,
            color_blend,
            model_resource_id: model_resource_id.as_ref().map(ModelReferenceType::id),
        }
    }

    pub fn new_default_mesh(mesh_type: DefaultMeshType, color: Color) -> Self {
        Self {
            color,
            color_blend: 1.0,
            model_resource_id: Some(ResourceTypeAndId {
                kind: ResourceType::from_raw(1),
                id: ResourceId::from_str(DEFAULT_MESH_GUIDS[mesh_type as usize]).unwrap(),
            }),
        }
    }
}
