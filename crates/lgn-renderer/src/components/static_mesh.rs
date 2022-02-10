use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;

use crate::resources::{DefaultMeshType, GpuUniformDataContext, MeshManager};

use super::MaterialComponent;
#[derive(Component)]
pub struct StaticMesh {
    pub color: Color,
    pub mesh_id: usize,
    pub num_vertices: u32,
    pub gpu_instance_id: u32,

    // GPU virtual addresses
    pub instance_va_table: u32,
    pub instance_color_va: u32,
    pub instance_transform_va: u32,
    pub instance_picking_data: u32,
    pub material_va: u32,
    pub mesh_description_va: u32,
}

impl StaticMesh {
    pub fn from_default_meshes(
        mesh_manager: &MeshManager,
        mesh_id: usize,
        color: Color,
        material: Option<&MaterialComponent>,
        data_context: &mut GpuUniformDataContext<'_>,
    ) -> Self {
        let gpu_instance_id = data_context.aquire_gpu_instance_id();

        let mut clamped_mesh_id = mesh_id as u32;
        if clamped_mesh_id > DefaultMeshType::RotationRing as u32 {
            clamped_mesh_id = 0;
        }

        let va_table_address = data_context
            .uniform_data
            .gpu_instance_va_table
            .ensure_index_allocated(gpu_instance_id) as u32;

        let instance_color_va = data_context
            .uniform_data
            .gpu_instance_color
            .ensure_index_allocated(gpu_instance_id) as u32;

        let instance_transform_va = data_context
            .uniform_data
            .gpu_instance_transform
            .ensure_index_allocated(gpu_instance_id) as u32;

        let picking_data_va = data_context
            .uniform_data
            .gpu_instance_picking_data
            .ensure_index_allocated(gpu_instance_id) as u32;

        Self {
            color,
            mesh_id,
            num_vertices: mesh_manager.mesh_from_id(clamped_mesh_id).num_vertices() as u32,
            gpu_instance_id,
            instance_va_table: va_table_address,
            instance_color_va,
            instance_transform_va,
            instance_picking_data: picking_data_va,
            mesh_description_va: mesh_manager.mesh_description_offset_from_id(clamped_mesh_id),
            material_va: if let Some(material) = material {
                material.gpu_offset()
            } else {
                u32::MAX
            },
        }
    }

    pub fn new_cpu_only(color: Color, mesh_id: usize, mesh_manager: &MeshManager) -> Self {
        let mut clamped_mesh_id = mesh_id as u32;
        if clamped_mesh_id > mesh_manager.max_id() as u32 {
            clamped_mesh_id = 0;
        }

        Self {
            color,
            mesh_id,
            num_vertices: mesh_manager.mesh_from_id(clamped_mesh_id).num_vertices() as u32,
            gpu_instance_id: u32::MAX,
            instance_va_table: u32::MAX,
            instance_color_va: u32::MAX,
            instance_transform_va: u32::MAX,
            instance_picking_data: u32::MAX,
            mesh_description_va: mesh_manager.mesh_description_offset_from_id(clamped_mesh_id),
            material_va: u32::MAX,
        }
    }
}
