use lgn_graphics_api::PagedBufferAllocation;

use super::{UnifiedStaticBuffer, UniformGPUDataUpdater};
use crate::{
    static_mesh_render_data::{MeshInfo, StaticMeshRenderData},
    Renderer,
};

pub struct MeshManager {
    static_buffer: UnifiedStaticBuffer,
    static_meshes: Vec<StaticMeshRenderData>,
    mesh_description_offsets: Vec<u32>,
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
            mesh_description_offsets: Vec::new(),
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
            vertex_data_size_in_bytes +=
                u64::from(mesh.size_in_bytes()) + std::mem::size_of::<MeshInfo>() as u64;
        }

        let static_allocation = self
            .static_buffer
            .allocate_segment(vertex_data_size_in_bytes);

        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
        let mut static_mesh_infos = Vec::with_capacity(meshes.len());
        let mut offset = static_allocation.offset();

        for mesh in &meshes {
            let (new_offset, mesh_info) = mesh.make_gpu_update_job(&mut updater, offset as u32);
            static_mesh_infos.push(mesh_info);
            offset = u64::from(new_offset);
        }

        let mut mesh_description_offsets = Vec::with_capacity(meshes.len());
        updater.add_update_jobs(&static_mesh_infos, offset);
        for (i, _mesh_info) in static_mesh_infos.into_iter().enumerate() {
            mesh_description_offsets
                .push(offset as u32 + (i * std::mem::size_of::<MeshInfo>()) as u32);
        }

        renderer.add_update_job_block(updater.job_blocks());
        self.static_meshes.append(&mut meshes);
        self.mesh_description_offsets
            .append(&mut mesh_description_offsets);
        self.allocations.push(static_allocation);
    }

    pub fn mesh_description_offset_from_id(&self, mesh_id: u32) -> u32 {
        if mesh_id < self.mesh_description_offsets.len() as u32 {
            self.mesh_description_offsets[mesh_id as usize]
        } else {
            0
        }
    }

    pub fn mesh_from_id(&self, mesh_id: u32) -> &StaticMeshRenderData {
        &self.static_meshes[mesh_id as usize]
    }

    pub fn mesh_indices_from_id(&self, mesh_id: u32) -> &Option<Vec<u32>> {
        &self.static_meshes[mesh_id as usize].indices
    }

    pub fn max_id(&self) -> usize {
        self.static_meshes.len()
    }
}
