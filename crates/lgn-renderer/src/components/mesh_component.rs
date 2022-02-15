use lgn_data_runtime::ResourceId;
use lgn_ecs::prelude::Component;

use crate::static_mesh_render_data::StaticMeshRenderData;

#[derive(Component)]
pub struct MeshComponent {
    pub submeshes: Vec<(ResourceId, StaticMeshRenderData)>,
}

impl MeshComponent {}
