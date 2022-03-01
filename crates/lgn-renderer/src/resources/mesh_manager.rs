use lgn_graphics_data::DefaultMeshType;
use lgn_math::Vec4;

use super::{StaticBufferAllocation, UnifiedStaticBuffer, UniformGPUDataUpdater};
use crate::{cgen::cgen_type::MeshDescription, components::Mesh, Renderer};

pub struct MeshMetaData {
    pub draw_call_count: u32,
    pub mesh_description_offset: u32,
    pub positions: Vec<Vec4>, // for AABB calculation
}

pub struct MeshManager {
    static_buffer: UnifiedStaticBuffer,
    static_meshes: Vec<MeshMetaData>,
    allocations: Vec<StaticBufferAllocation>,
}

pub(crate) const DEFAULT_MESH_GUIDS: [(DefaultMeshType, &str); 11] = [
    (
        DefaultMeshType::Plane,
        "f6d83574-3098-4c90-8782-9bc8df96c58e",
    ),
    (
        DefaultMeshType::Cube,
        "c75cd9e7-57f5-4469-b400-8f5b3a071f9e",
    ),
    (
        DefaultMeshType::Pyramid,
        "05dcb241-f663-4b07-8b02-e1f66e23e01c",
    ),
    (
        DefaultMeshType::WireframeCube,
        "a897c7ae-290b-4a90-91c2-78e1a00e5548",
    ),
    (
        DefaultMeshType::GroundPlane,
        "582fee7b-fcc6-4351-ae38-7b0e149b48ca",
    ),
    (
        DefaultMeshType::Torus,
        "bb492db3-0985-4515-9f93-0cc5c6ecf8fc",
    ),
    (
        DefaultMeshType::Cone,
        "c0d167de-bc48-40b6-92b9-1580c14116da",
    ),
    (
        DefaultMeshType::Cylinder,
        "5b74aae6-83e7-4e36-8636-0572aaf6fdfa",
    ),
    (
        DefaultMeshType::Sphere,
        "f5a4b115-1478-4b2e-93f8-585214dec334",
    ),
    (
        DefaultMeshType::Arrow,
        "53c027f6-e349-44fa-b921-178f84513df7",
    ),
    (
        DefaultMeshType::RotationRing,
        "7038f75c-04ca-438e-bc67-8fe397c9dbf6",
    ),
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
        debug_assert_eq!(default_meshes.len(), DEFAULT_MESH_GUIDS.len());

        mesh_manager.add_meshes(renderer, &default_meshes);
        mesh_manager
    }

    pub fn add_meshes(&mut self, renderer: &Renderer, meshes: &[Mesh]) -> Vec<u32> {
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
