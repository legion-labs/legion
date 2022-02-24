use lgn_graphics_api::PagedBufferAllocation;
use lgn_math::Vec4;
use strum::EnumIter;

use super::{UnifiedStaticBuffer, UniformGPUDataUpdater};
use crate::{cgen::cgen_type::MeshDescription, components::Mesh, Renderer};

pub struct MeshMetaData {
    pub draw_call_count: u32,
    pub mesh_description_offset: u32,
    pub positions: Vec<Vec4>, // for AABB calculation
}

pub struct MeshManager {
    static_buffer: UnifiedStaticBuffer,
    static_meshes: Vec<MeshMetaData>,
    allocations: Vec<PagedBufferAllocation>,
}

impl Drop for MeshManager {
    fn drop(&mut self) {
        while let Some(allocation) = self.allocations.pop() {
            self.static_buffer.free_segment(allocation);
        }
    }
}

#[derive(EnumIter)]
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

pub const DEFAULT_MESH_GUIDS: [&str; 11] = [
    "f6d83574-3098-4c90-8782-9bc8df96c58e",
    "c75cd9e7-57f5-4469-b400-8f5b3a071f9e",
    "05dcb241-f663-4b07-8b02-e1f66e23e01c",
    "a897c7ae-290b-4a90-91c2-78e1a00e5548",
    "582fee7b-fcc6-4351-ae38-7b0e149b48ca",
    "bb492db3-0985-4515-9f93-0cc5c6ecf8fc",
    "c0d167de-bc48-40b6-92b9-1580c14116da",
    "5b74aae6-83e7-4e36-8636-0572aaf6fdfa",
    "f5a4b115-1478-4b2e-93f8-585214dec334",
    "53c027f6-e349-44fa-b921-178f84513df7",
    "7038f75c-04ca-438e-bc67-8fe397c9dbf6",
];

impl MeshManager {
    pub fn new(renderer: &Renderer) -> Self {
        let static_buffer = renderer.static_buffer().clone();

        let mut mesh_manager = Self {
            static_buffer,
            static_meshes: Vec::new(),
            allocations: Vec::new(),
        };

        // Keep consistent with DefaultMeshType
        let default_meshes = vec![
            Mesh::new_plane(1.0),
            Mesh::new_cube(0.5),
            Mesh::new_pyramid(0.5, 1.0),
            Mesh::new_wireframe_cube(1.0),
            Mesh::new_ground_plane(6, 5, 0.25),
            Mesh::new_torus(0.1, 32, 0.5, 128),
            Mesh::new_cone(0.25, 1.0, 32),
            Mesh::new_cylinder(0.25, 1.0, 32),
            Mesh::new_sphere(0.25, 64, 64),
            Mesh::new_arrow(),
            Mesh::new_torus(0.01, 8, 0.5, 128),
        ];

        mesh_manager.add_meshes(renderer, &default_meshes);
        mesh_manager
    }

    pub fn add_meshes(&mut self, renderer: &Renderer, meshes: &Vec<Mesh>) -> Vec<u32> {
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

    pub fn max_id(&self) -> usize {
        self.static_meshes.len()
    }
}
