use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
#[derive(Component)]
pub struct StaticMesh {
    pub mesh_id: usize,
    pub color: Color,
    pub offset: u64,
}
