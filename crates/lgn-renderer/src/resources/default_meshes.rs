use lgn_graphics_api::PagedBufferAllocation;

use super::{UnifiedStaticBuffer, UniformGPUDataUpdater};
use crate::{static_mesh_render_data::StaticMeshRenderData, Renderer};

pub struct DefaultMeshes {
    static_buffer: UnifiedStaticBuffer,
    static_meshes: Vec<StaticMeshRenderData>,
    static_mesh_offsets: Vec<u32>,
    static_allocation: Option<PagedBufferAllocation>,
}

impl Drop for DefaultMeshes {
    fn drop(&mut self) {
        self.static_buffer
            .free_segment(self.static_allocation.take().unwrap());
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

impl DefaultMeshes {
    pub fn new(renderer: &Renderer) -> Self {
        // Keep consistent with DefaultMeshId
        let static_meshes = vec![
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
            StaticMeshRenderData::new_gltf(String::from(
                "C:/work/glTF-Sample-Models/2.0/FlightHelmet/glTF/FlightHelmet.gltf",
            )),
        ];

        let mut vertex_data_size_in_bytes = 0;
        for mesh in &static_meshes {
            vertex_data_size_in_bytes += mesh.vertices.len() as u64 * 4;
        }

        let static_buffer = renderer.static_buffer().clone();
        let static_allocation = static_buffer.allocate_segment(vertex_data_size_in_bytes);

        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
        let mut static_mesh_offsets = Vec::with_capacity(static_meshes.len());
        let mut offset = static_allocation.offset();

        for mesh in &static_meshes {
            static_mesh_offsets.push(offset as u32);
            updater.add_update_jobs(&mesh.vertices, offset);
            offset += mesh.vertices.len() as u64 * 4;
        }

        renderer.add_update_job_block(updater.job_blocks());

        Self {
            static_buffer,
            static_meshes,
            static_mesh_offsets,
            static_allocation: Some(static_allocation),
        }
    }

    pub fn mesh_offset_from_id(&self, mesh_id: u32) -> u32 {
        if mesh_id < self.static_mesh_offsets.len() as u32 {
            self.static_mesh_offsets[mesh_id as usize]
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
}
