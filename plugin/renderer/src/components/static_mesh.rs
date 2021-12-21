use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
#[derive(Component)]
pub struct StaticMesh {
    pub mesh_id: usize,
    pub color: Color,
    pub vertex_offset: u32,
    pub num_verticies: u32,
    pub world_offset: u32,
    pub picking_id: u32,
}
