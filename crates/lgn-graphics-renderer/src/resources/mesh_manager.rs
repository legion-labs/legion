use std::num::NonZeroU32;

use lgn_graphics_api::ResourceUsage;
use lgn_math::{Vec3, Vec4};
use strum::EnumIter;

use super::{StaticBufferAllocation, UnifiedStaticBuffer, UpdateUnifiedStaticBufferCommand};
use crate::{
    components::{Mesh, MeshTopology},
    core::RenderCommandBuilder,
};

#[derive(Clone, Copy)]
pub struct MeshId(u32);

pub struct MeshMetaData {
    pub vertex_count: NonZeroU32,
    pub index_count: NonZeroU32,
    pub index_offset: u32,            // static_buffer_index_offset
    pub mesh_description_offset: u32, // static_buffer_mesh_description_offset
    pub positions: Vec<Vec3>,         // for AABB calculation
    pub bounding_sphere: Vec4,
    pub topology: MeshTopology,
    _allocation: StaticBufferAllocation,
}

pub struct MeshManager {
    gpu_heap: UnifiedStaticBuffer,
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
    pub fn new(gpu_heap: &UnifiedStaticBuffer) -> Self {
        Self {
            gpu_heap: gpu_heap.clone(),
            default_mesh_ids: Vec::new(),
            static_meshes: Vec::new(),
        }
    }

    pub fn initialize_default_meshes(&mut self, render_commands: &mut RenderCommandBuilder) {
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
            let mesh_id = self.add_mesh(render_commands, default_mesh);
            self.default_mesh_ids.push(mesh_id);
        }
    }

    pub fn add_mesh(&mut self, render_commands: &mut RenderCommandBuilder, mesh: &Mesh) -> MeshId {
        let (buf, index_byte_offset) = mesh.pack_gpu_data();
        assert_eq!(index_byte_offset % 4, 0);

        let allocation = self
            .gpu_heap
            .allocate(buf.len() as u64, ResourceUsage::AS_SHADER_RESOURCE);

        let allocation_offset = u32::try_from(allocation.byte_offset()).unwrap();
        assert_eq!(allocation_offset % 4, 0);

        let index_offset =
            (allocation_offset + index_byte_offset) / std::mem::size_of::<u16>() as u32;
        assert_eq!(index_offset % 4, 0);

        let mesh_id = self.static_meshes.len();

        self.static_meshes.push(MeshMetaData {
            vertex_count: NonZeroU32::new(mesh.num_vertices() as u32).unwrap(),
            index_count: NonZeroU32::new(mesh.num_indices() as u32).unwrap(),
            index_offset,
            mesh_description_offset: allocation_offset,
            positions: mesh.positions.clone(),
            bounding_sphere: mesh.bounding_sphere,
            topology: mesh.topology,
            _allocation: allocation.clone(),
        });

        render_commands.push(UpdateUnifiedStaticBufferCommand {
            src_buffer: buf,
            dst_offset: allocation.byte_offset(),
        });

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
