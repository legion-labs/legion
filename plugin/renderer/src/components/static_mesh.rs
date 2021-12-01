use graphics_data::Color;
use legion_ecs::prelude::*;
#[derive(Component)]
pub struct StaticMesh {
    pub mesh_id: usize,
    pub color: Color,
}
