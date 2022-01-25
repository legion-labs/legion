use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
use lgn_tracing::span_fn;

use crate::resources::{DefaultMaterialType, DefaultMeshes};
#[derive(Component)]

pub struct StaticMesh {
    pub mesh_id: usize,
    pub color: Color,
    pub vertex_offset: u32,
    pub num_vertices: u32,
    pub world_offset: u32,
    pub material_offset: u32,
    pub picking_id: u32,
    pub material_type: DefaultMaterialType,
}

impl StaticMesh {
    #[span_fn]
    pub fn from_default_meshes(
        default_meshes: &DefaultMeshes,
        mesh_id: usize,
        color: Color,
        material_type: DefaultMaterialType,
    ) -> Self {
        Self {
            mesh_id,
            color,
            vertex_offset: default_meshes.mesh_offset_from_id(mesh_id as u32),
            num_vertices: default_meshes.mesh_from_id(mesh_id as u32).num_vertices() as u32,
            world_offset: 0,
            material_offset: 0,
            picking_id: 0,
            material_type,
        }
    }
}
