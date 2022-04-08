use lgn_math::Vec4;
use strum::EnumIter;

use super::{StaticBufferAllocation, UnifiedStaticBufferAllocator, UniformGPUDataUpdater};
use crate::{cgen::cgen_type::MeshDescription, components::Mesh, Renderer};

#[derive(Clone, Copy)]
pub struct MeshId(u32);

pub struct MeshMetaData {
    pub vertex_count: u32,
    pub index_count: u32,
    pub index_offset: u32,
    pub mesh_description_offset: u32,
    pub positions: Vec<Vec4>, // for AABB calculation
    pub bounding_sphere: Vec4,
    allocation: StaticBufferAllocation,
}

pub struct MeshManager {
    allocator: UnifiedStaticBufferAllocator,
    default_mesh_ids: Vec<MeshId>,
    static_meshes: Vec<MeshMetaData>,
}

#[derive(EnumIter, Clone, Copy)]
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
        let allocator = renderer.static_buffer_allocator();

        let mut mesh_manager = Self {
            allocator: allocator.clone(),
            default_mesh_ids: Vec::new(),
            static_meshes: Vec::new(),
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

        for default_mesh in &default_meshes {
            let mesh_id = mesh_manager.add_mesh(renderer, default_mesh);
            mesh_manager.default_mesh_ids.push(mesh_id);
        }

        mesh_manager
    }

    pub fn add_mesh(&mut self, renderer: &Renderer, mesh: &Mesh) -> MeshId {
        let mut vertex_data_size_in_bytes =
            u64::from(mesh.size_in_bytes()) + std::mem::size_of::<MeshDescription>() as u64;

        let allocation = self.allocator.allocate_segment(vertex_data_size_in_bytes);

        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
        let mut offset = allocation.offset();
        let mut mesh_meta_datas = Vec::new();
        let mesh_id = self.static_meshes.len();

        let (_, mesh_info_offset, index_offset) =
            mesh.make_gpu_update_job(&mut updater, offset as u32);

        mesh_meta_datas.push(MeshMetaData {
            vertex_count: mesh.num_vertices() as u32,
            index_count: mesh.num_indices() as u32,
            index_offset,
            mesh_description_offset: mesh_info_offset,
            positions: mesh.positions.iter().map(|v| v.extend(1.0)).collect(),
            bounding_sphere: mesh.bounding_sphere,
            allocation,
        });

        renderer.add_update_job_block(updater.job_blocks());
        self.static_meshes.append(&mut mesh_meta_datas);

        MeshId(u32::try_from(mesh_id).unwrap())
    }

    pub fn get_default_mesh_id(&self, default_mesh_type: DefaultMeshType) -> MeshId {
        self.default_mesh_ids[default_mesh_type as usize]
    }

    pub fn get_default_mesh(&self, default_mesh_type: DefaultMeshType) -> &MeshMetaData {
        let mesh_id = self.get_default_mesh_id(default_mesh_type);
        self.get_mesh_meta_data(mesh_id)
    }

    pub fn get_mesh_meta_data(&self, mesh_id: MeshId) -> &MeshMetaData {
        &self.static_meshes[mesh_id.0 as usize]
    }
}
