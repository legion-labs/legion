use lgn_math::{Vec3, Vec4};
use strum::EnumIter;

use super::{
    StaticBufferAllocation, UnifiedStaticBufferAllocator, UpdateUnifiedStaticBufferCommand,
};
use crate::{components::Mesh, core::RenderCommandBuilder};

#[derive(Clone, Copy)]
pub struct MeshId(u32);

pub struct MeshMetaData {
    pub vertex_count: u32,
    pub index_count: u32,
    pub index_offset: u32,
    pub mesh_description_offset: u32,
    pub positions: Vec<Vec3>, // for AABB calculation
    pub bounding_sphere: Vec4,
    _allocation: StaticBufferAllocation,
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
    pub fn new(allocator: &UnifiedStaticBufferAllocator) -> Self {
        Self {
            allocator: allocator.clone(),
            default_mesh_ids: Vec::new(),
            static_meshes: Vec::new(),
        }
    }

    pub fn initialize_default_meshes(&mut self, mut render_commands: RenderCommandBuilder) {
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
            let mesh_id = self.add_mesh(&mut render_commands, default_mesh);
            self.default_mesh_ids.push(mesh_id);
        }
    }

    pub fn add_mesh(&mut self, render_commands: &mut RenderCommandBuilder, mesh: &Mesh) -> MeshId {
        let (buf, index_offset) = mesh.pack_gpu_data();
        let allocation = self.allocator.allocate(buf.len() as u64);
        let allocation_offset = u32::try_from(allocation.byte_offset()).unwrap();

        self.static_meshes.push(MeshMetaData {
            vertex_count: mesh.num_vertices() as u32,
            index_count: mesh.num_indices() as u32,
            index_offset: allocation_offset + index_offset,
            mesh_description_offset: allocation_offset,
            positions: mesh.positions.clone(),
            bounding_sphere: mesh.bounding_sphere,
            _allocation: allocation.clone(),
        });

        render_commands.push(UpdateUnifiedStaticBufferCommand {
            src_buffer: buf,
            dst_offset: allocation.byte_offset(),
        });

        MeshId(u32::try_from(self.static_meshes.len() - 1).unwrap())
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
