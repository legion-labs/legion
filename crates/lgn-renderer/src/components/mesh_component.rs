use lgn_ecs::prelude::Component;

use crate::static_mesh_render_data::StaticMeshRenderData;

#[derive(Component)]
pub struct MeshComponent {
    pub submeshes: Vec<StaticMeshRenderData>,
}

impl MeshComponent {}
