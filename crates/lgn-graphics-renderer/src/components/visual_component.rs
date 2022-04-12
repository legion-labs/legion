use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;

use crate::features::mesh_feature::mesh_feature::*;

#[derive(Component)]
pub struct VisualComponent {
    color: Color,
    color_blend: f32,
    model_resource_id: Option<ResourceTypeAndId>,
}

impl VisualComponent {
    pub fn new(
        model_resource_id: Option<ResourceTypeAndId>,
        color: Color,
        color_blend: f32,
    ) -> Self {
        Self {
            color,
            color_blend,
            model_resource_id,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn color_blend(&self) -> f32 {
        self.color_blend
    }

    pub fn model_resource_id(&self) -> Option<&ResourceTypeAndId> {
        self.model_resource_id.as_ref()
    }
}
