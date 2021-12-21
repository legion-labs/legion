use crate::resources::DefaultMeshes;
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

impl StaticMesh {
    pub fn from_default_meshes(
        default_meshes: &DefaultMeshes,
        mesh_id: usize,
        color: Color,
    ) -> Self {
        Self {
            mesh_id,
            color,
            vertex_offset: default_meshes.mesh_offset_from_id(mesh_id as u32),
            num_verticies: default_meshes.mesh_from_id(mesh_id as u32).num_vertices() as u32,
            world_offset: 0,
            picking_id: 0,
        }
    }
}
