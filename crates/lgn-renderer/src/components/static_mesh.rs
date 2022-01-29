use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
use lgn_tracing::span_fn;

use crate::resources::{DefaultMaterialType, DefaultMeshType, DefaultMeshes};
#[derive(Component)]
pub struct StaticMesh {
    pub mesh_id: usize,
    pub color: Color,
    pub num_vertices: u32,
    pub picking_id: u32,
    pub material_type: DefaultMaterialType,

    // GPU instance data id and static buffer offsets
    pub gpu_instance_id: u32,
    pub va_table_address: u32,
    pub instance_color_va: u32,
    pub world_transform_va: u32,
    pub vertex_buffer_va: u32,
    pub picking_data_va: u32,
}

impl StaticMesh {
    #[span_fn]
    pub fn from_default_meshes(
        default_meshes: &DefaultMeshes,
        mesh_id: usize,
        color: Color,
        material_type: DefaultMaterialType,
    ) -> Self {
        let mut clamped_mesh_id = mesh_id as u32;
        if clamped_mesh_id > DefaultMeshType::Helmet_Lenses as u32 {
            clamped_mesh_id = 0;
        }
        Self {
            mesh_id,
            color,
            num_vertices: default_meshes.mesh_from_id(clamped_mesh_id).num_vertices() as u32,
            picking_id: 0,
            material_type,
            gpu_instance_id: u32::MAX,
            va_table_address: u32::MAX,
            instance_color_va: u32::MAX,
            world_transform_va: u32::MAX,
            vertex_buffer_va: default_meshes.mesh_offset_from_id(clamped_mesh_id),
            picking_data_va: u32::MAX,
        }
    }
}
