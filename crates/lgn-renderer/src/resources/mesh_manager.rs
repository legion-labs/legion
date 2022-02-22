use lgn_graphics_api::PagedBufferAllocation;
use lgn_math::Vec4;

use super::{UnifiedStaticBuffer, UniformGPUDataUpdater};
use crate::{
    cgen::{self, cgen_type::MeshDescription},
    components::SubMesh,
    static_mesh_render_data::StaticMeshRenderData,
    Renderer,
};

pub struct MeshMetaData {
    pub draw_call_count: u32,
    pub mesh_description_offset: u32,
    pub positions: Vec<Vec4>, // for AABB calculation
}

pub struct MeshManager {
    static_buffer: UnifiedStaticBuffer,
    static_meshes: Vec<MeshMetaData>,
    default_mesh_id: u32,
    allocations: Vec<PagedBufferAllocation>,
}

impl Drop for MeshManager {
    fn drop(&mut self) {
        while let Some(allocation) = self.allocations.pop() {
            self.static_buffer.free_segment(allocation);
        }
    }
}

pub enum DefaultMeshType {
    Plane = 0,
    Cube,
    Pyramid,
    WireframeCube,
    GroundPlane,
    Torus,
    Cone,
    Cylinder,
    Sphere,
    Arrow,
    RotationRing,
}

impl MeshManager {
    pub fn new(renderer: &Renderer) -> Self {
        let static_buffer = renderer.static_buffer().clone();

        let mut mesh_manager = Self {
            static_buffer,
            static_meshes: Vec::new(),
            default_mesh_id: 1, // Cube
            allocations: Vec::new(),
        };

        // Keep consistent with DefaultMeshType
        let default_meshes = vec![
            StaticMeshRenderData::new_plane(1.0),
            StaticMeshRenderData::new_cube(0.5),
            StaticMeshRenderData::new_pyramid(0.5, 1.0),
            StaticMeshRenderData::new_wireframe_cube(1.0),
            StaticMeshRenderData::new_ground_plane(6, 5, 0.25),
            StaticMeshRenderData::new_torus(0.1, 32, 0.5, 128),
            StaticMeshRenderData::new_cone(0.25, 1.0, 32),
            StaticMeshRenderData::new_cylinder(0.25, 1.0, 32),
            StaticMeshRenderData::new_sphere(0.25, 64, 64),
            StaticMeshRenderData::new_arrow(),
            StaticMeshRenderData::new_torus(0.01, 8, 0.5, 128),
        ];

        mesh_manager.add_meshes(renderer, default_meshes);
        mesh_manager
    }

    pub fn add_meshes(&mut self, renderer: &Renderer, mut meshes: Vec<StaticMeshRenderData>) {
        if meshes.is_empty() {
            return;
        }
        let mut vertex_data_size_in_bytes = 0;
        for mesh in &meshes {
            vertex_data_size_in_bytes += u64::from(mesh.size_in_bytes())
                + std::mem::size_of::<cgen::cgen_type::MeshDescription>() as u64;
        }

        let static_allocation = self
            .static_buffer
            .allocate_segment(vertex_data_size_in_bytes);

        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
        let mut offset = static_allocation.offset();
        let mut mesh_meta_datas = Vec::new();

        for mesh in &meshes {
            let (new_offset, mesh_info_offset) =
                mesh.make_gpu_update_job(&mut updater, offset as u32);
            mesh_meta_datas.push(MeshMetaData {
                draw_call_count: mesh.num_vertices() as u32,
                mesh_description_offset: mesh_info_offset,
                positions: mesh.positions.as_ref().unwrap().clone(),
            });
            offset = u64::from(new_offset);
        }

        renderer.add_update_job_block(updater.job_blocks());
        self.static_meshes.append(&mut mesh_meta_datas);
        self.allocations.push(static_allocation);
    }

    pub fn add_mesh_components(&mut self, renderer: &Renderer, meshes: &Vec<SubMesh>) -> Vec<u32> {
        if meshes.is_empty() {
            return Vec::new();
        }
        let mut mesh_ids = Vec::new();
        let mut vertex_data_size_in_bytes = 0;
        for mesh in meshes {
            vertex_data_size_in_bytes +=
                u64::from(mesh.size_in_bytes()) + std::mem::size_of::<MeshDescription>() as u64;
        }

        let static_allocation = self
            .static_buffer
            .allocate_segment(vertex_data_size_in_bytes);

        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
        let mut offset = static_allocation.offset();
        let mut mesh_meta_datas = Vec::new();

        for mesh in meshes {
            mesh_ids.push(self.static_meshes.len() as u32);
            let (new_offset, mesh_info_offset) =
                mesh.make_gpu_update_job(&mut updater, offset as u32);
            mesh_meta_datas.push(MeshMetaData {
                draw_call_count: mesh.num_vertices() as u32,
                mesh_description_offset: mesh_info_offset,
                positions: mesh.positions.as_ref().unwrap().clone(),
            });
            offset = u64::from(new_offset);
        }

        renderer.add_update_job_block(updater.job_blocks());
        self.static_meshes.append(&mut mesh_meta_datas);
        self.allocations.push(static_allocation);

        mesh_ids
    }

    pub fn get_mesh_meta_data(&self, mesh_id: u32) -> &MeshMetaData {
        &self.static_meshes[mesh_id as usize]
    }

    //pub fn mesh_from_id(&self, mesh_id: u32) -> &StaticMeshRenderData {
    //    &self.static_meshes[mesh_id as usize]
    //}

    pub fn max_id(&self) -> usize {
        self.static_meshes.len()
    }
}
