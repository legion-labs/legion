use std::{num::NonZeroU32, ops::Mul, sync::Arc};

use lgn_graphics_api::ResourceUsage;
use lgn_math::{Mat4, Vec2, Vec3, Vec4};
use lgn_utils::memory::round_size_up_to_alignment_u32;
use parking_lot::{lock_api::RwLockReadGuard, RawRwLock, RwLock};
use slotmap::{DefaultKey, SlotMap};
use strum::{EnumCount, EnumIter};

use super::{StaticBufferAllocation, UnifiedStaticBuffer, UpdateUnifiedStaticBufferCommand};
use crate::{
    cgen::cgen_type::{MeshAttribMask, MeshDescription},
    core::{
        BinaryWriter, GpuUploadManager, RenderCommandBuilder, TransferError, UploadGPUBuffer,
        UploadGPUResource,
    },
    DOWN_VECTOR, UP_VECTOR,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshTopology {
    LineList,
    TriangleList,
}

pub struct Mesh {
    pub indices: Vec<u16>,
    pub positions: Vec<Vec3>,
    pub normals: Option<Vec<Vec3>>,
    pub tangents: Option<Vec<Vec4>>,
    pub tex_coords: Option<Vec<Vec2>>,
    pub colors: Option<Vec<[u8; 4]>>,
    pub topology: MeshTopology,
    pub bounding_sphere: Vec4,
}

impl From<&lgn_graphics_data::runtime::Mesh> for Mesh {
    fn from(mesh_data: &lgn_graphics_data::runtime::Mesh) -> Self {
        assert!(!mesh_data.indices.is_empty());
        assert!(!mesh_data.positions.is_empty());
        assert_eq!(mesh_data.indices.len() % 3, 0); // triangle list
        Mesh {
            indices: mesh_data.indices.clone(),
            positions: mesh_data.positions.clone(),
            normals: if !mesh_data.normals.is_empty() {
                Some(mesh_data.normals.clone())
            } else {
                None
            },
            tangents: if !mesh_data.tangents.is_empty() {
                Some(mesh_data.tangents.clone())
            } else {
                None
            },
            tex_coords: if !mesh_data.tex_coords.is_empty() {
                Some(mesh_data.tex_coords.clone())
            } else {
                None
            },
            colors: if !mesh_data.colors.is_empty() {
                Some(mesh_data.colors.iter().map(|v| Into::into(*v)).collect())
            } else {
                None
            },
            bounding_sphere: Mesh::calculate_bounding_sphere(&mesh_data.positions),
            topology: MeshTopology::TriangleList,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RenderMeshId(DefaultKey);

pub struct RenderMesh {
    pub vertex_count: NonZeroU32,
    pub index_count: NonZeroU32,
    pub index_offset: u32,            // static_buffer_index_offset
    pub mesh_description_offset: u32, // static_buffer_mesh_description_offset
    pub positions: Vec<Vec3>,         // for AABB calculation
    pub bounding_sphere: Vec4,
    pub topology: MeshTopology,
    _allocation: StaticBufferAllocation,
}

type RenderMeshSlotMap = SlotMap<DefaultKey, Box<RenderMesh>>;

struct Inner {
    gpu_heap: UnifiedStaticBuffer,
    gpu_upload_manager: GpuUploadManager,
    default_mesh_ids: Vec<RenderMeshId>,
    render_meshes: RwLock<RenderMeshSlotMap>,
}

#[derive(Clone)]
pub struct MeshManager {
    inner: Arc<Inner>,
}

#[derive(EnumCount, EnumIter, Clone, Copy)]
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

pub struct RenderMeshReader<'a> {
    inner: Arc<Inner>,
    render_meshes_guard: RwLockReadGuard<'a, RawRwLock, RenderMeshSlotMap>,
}

impl<'a> RenderMeshReader<'a> {
    pub fn get_default_mesh_id(&self, default_mesh_type: DefaultMeshType) -> RenderMeshId {
        self.inner.default_mesh_ids[default_mesh_type as usize]
    }

    pub fn get_default_mesh(&self, default_mesh_type: DefaultMeshType) -> &RenderMesh {
        let mesh_id = self.get_default_mesh_id(default_mesh_type);
        self.get_render_mesh(mesh_id)
    }

    pub fn get_render_mesh(&self, mesh_id: RenderMeshId) -> &RenderMesh {
        self.try_get_render_mesh(mesh_id).unwrap()
    }

    pub fn try_get_render_mesh(&self, mesh_id: RenderMeshId) -> Option<&RenderMesh> {
        self.render_meshes_guard.get(mesh_id.0).map(Box::as_ref)
    }
}

impl MeshManager {
    pub fn new(
        gpu_heap: &UnifiedStaticBuffer,
        gpu_upload_manager: &GpuUploadManager,
        render_commands: &mut RenderCommandBuilder,
    ) -> Self {
        let mut default_meshes = Self::create_default_meshes();

        let mut default_render_meshes = default_meshes
            .drain(..)
            .map(|mesh| Self::create_render_mesh(gpu_heap, render_commands, &mesh))
            .collect::<Vec<_>>();

        let mut default_mesh_ids = Vec::new();
        let mut render_meshes = SlotMap::new();
        for default_render_mesh in default_render_meshes.drain(..) {
            let key = render_meshes.insert(Box::new(default_render_mesh));
            default_mesh_ids.push(RenderMeshId(key));
        }

        Self {
            inner: Arc::new(Inner {
                gpu_heap: gpu_heap.clone(),
                gpu_upload_manager: gpu_upload_manager.clone(),
                default_mesh_ids,
                render_meshes: RwLock::new(render_meshes),
            }),
        }
    }

    pub async fn install_mesh(&self, mesh: Mesh) -> Result<RenderMeshId, TransferError> {
        let render_mesh = Self::async_create_render_mesh(
            &self.inner.gpu_heap,
            &self.inner.gpu_upload_manager,
            &mesh,
        )
        .await?;

        let mut render_meshes = self.inner.render_meshes.write();

        let id = render_meshes.insert(Box::new(render_mesh));

        Ok(RenderMeshId(id))
    }

    pub fn get_default_mesh_id(&self, default_mesh_type: DefaultMeshType) -> RenderMeshId {
        self.inner.default_mesh_ids[default_mesh_type as usize]
    }

    // pub fn get_default_mesh(&self, default_mesh_type: DefaultMeshType) -> RenderMeshGuard<'_> {
    //     let mesh_id = self.get_default_mesh_id(default_mesh_type);
    //     self.get_render_mesh(mesh_id)
    // }

    // pub fn get_render_mesh(&self, mesh_id: RenderMeshId) -> RenderMeshGuard<'_> {
    //     self.try_get_render_mesh(mesh_id).unwrap()
    // }

    // pub fn try_get_render_mesh(&self, mesh_id: RenderMeshId) -> Option<RenderMeshGuard<'_>> {
    //     let render_meshes_guard = self.inner.render_meshes.read();

    //     RenderMeshGuard{
    //         render_meshes_guard
    //     }
    // }

    pub fn read(&self) -> RenderMeshReader<'_> {
        let render_meshes_guard = self.inner.render_meshes.read();
        RenderMeshReader {
            inner: self.inner.clone(),
            render_meshes_guard,
        }
    }

    fn create_default_meshes() -> Vec<Mesh> {
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

        default_meshes
    }

    fn create_render_mesh(
        gpu_heap: &UnifiedStaticBuffer,
        render_commands: &mut RenderCommandBuilder,
        mesh: &Mesh,
    ) -> RenderMesh {
        let (buf, index_byte_offset) = mesh.pack_gpu_data();
        assert_eq!(index_byte_offset % 4, 0);

        let allocation = gpu_heap.allocate(buf.len() as u64, ResourceUsage::AS_SHADER_RESOURCE);

        let allocation_offset = u32::try_from(allocation.byte_offset()).unwrap();
        assert_eq!(allocation_offset % 4, 0);

        let index_offset =
            (allocation_offset + index_byte_offset) / std::mem::size_of::<u16>() as u32;
        assert_eq!(index_offset % 4, 0);

        let render_mesh = RenderMesh {
            vertex_count: NonZeroU32::new(mesh.num_vertices() as u32).unwrap(),
            index_count: NonZeroU32::new(mesh.num_indices() as u32).unwrap(),
            index_offset,
            mesh_description_offset: allocation_offset,
            positions: mesh.positions.clone(),
            bounding_sphere: mesh.bounding_sphere,
            topology: mesh.topology,
            _allocation: allocation.clone(),
        };

        render_commands.push(UpdateUnifiedStaticBufferCommand {
            src_buffer: buf,
            dst_offset: allocation.byte_offset(),
        });

        render_mesh
    }

    async fn async_create_render_mesh(
        gpu_heap: &UnifiedStaticBuffer,
        gpu_transfer: &GpuUploadManager,
        mesh: &Mesh,
    ) -> Result<RenderMesh, TransferError> {
        let (buf, index_byte_offset) = mesh.pack_gpu_data();
        assert_eq!(index_byte_offset % 4, 0);

        let allocation = gpu_heap.allocate(buf.len() as u64, ResourceUsage::AS_SHADER_RESOURCE);

        let allocation_offset = u32::try_from(allocation.byte_offset()).unwrap();
        assert_eq!(allocation_offset % 4, 0);

        let index_offset =
            (allocation_offset + index_byte_offset) / std::mem::size_of::<u16>() as u32;
        assert_eq!(index_offset % 4, 0);

        let render_mesh = RenderMesh {
            vertex_count: NonZeroU32::new(mesh.num_vertices() as u32).unwrap(),
            index_count: NonZeroU32::new(mesh.num_indices() as u32).unwrap(),
            index_offset,
            mesh_description_offset: allocation_offset,
            positions: mesh.positions.clone(),
            bounding_sphere: mesh.bounding_sphere,
            topology: mesh.topology,
            _allocation: allocation.clone(),
        };

        gpu_transfer.async_upload(UploadGPUResource::Buffer(UploadGPUBuffer {
            src_data: buf,
            dst_buffer: allocation.buffer().clone(),
            dst_offset: allocation.byte_offset(),
        }))?;

        Ok(render_mesh)
    }
}

const DEFAULT_ALIGNMENT: usize = 64;
const DEFAULT_MESH_VERTEX_SIZE: usize = 12; // pos + normal + color + tex_coord = 3 + 3 + 4 + 2

fn byteadressbuffer_align(size: u32) -> u32 {
    round_size_up_to_alignment_u32(size, DEFAULT_ALIGNMENT as u32)
}

fn add_vertex_data(vertex_data: &mut Vec<f32>, pos: Vec3, normal: Vec3) {
    vertex_data.append(&mut vec![
        pos.x, pos.y, pos.z, normal.x, normal.y, normal.z, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]);
}

impl Mesh {
    pub fn get_mesh_attrib_mask(&self) -> MeshAttribMask {
        let mut format = MeshAttribMask::empty();
        if self.normals.is_some() {
            format |= MeshAttribMask::NORMAL;
        }
        if self.tangents.is_some() {
            format |= MeshAttribMask::TANGENT;
        }
        if self.tex_coords.is_some() {
            format |= MeshAttribMask::TEX_COORD;
        }
        if self.colors.is_some() {
            format |= MeshAttribMask::COLOR;
        }
        format
    }

    pub fn calculate_bounding_sphere(positions: &[Vec3]) -> Vec4 {
        let mut min_bound = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_bound = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for position in positions {
            min_bound = min_bound.min(*position);
            max_bound = max_bound.max(*position);
        }

        let delta = max_bound - min_bound;
        let mid_point = min_bound + delta * 0.5;

        let mut max_length: f32 = 0.0;
        for position in positions {
            let delta = *position - mid_point;
            let length = delta.abs().length();

            if length > max_length {
                max_length = length;
            }
        }
        mid_point.extend(max_length)
    }

    pub fn pack_gpu_data(&self) -> (Vec<u8>, u32) {
        let indice_size = (self.indices.len() * std::mem::size_of::<u16>()) as u32;
        let position_size = (std::mem::size_of::<Vec3>() * self.positions.len()) as u32;
        let normal_size = self.normals.as_ref().map_or(0, |stream| {
            (std::mem::size_of::<u32>() * stream.len()) as u32
        });
        let tangent_size = self.tangents.as_ref().map_or(0, |stream| {
            (std::mem::size_of::<u32>() * stream.len()) as u32
        });
        let texcoord_size = self.tex_coords.as_ref().map_or(0, |stream| {
            (std::mem::size_of::<Vec2>() * stream.len()) as u32
        });
        let color_size = self.colors.as_ref().map_or(0, |stream| {
            (std::mem::size_of::<[u8; 4]>() * stream.len()) as u32
        });

        let indice_offset = byteadressbuffer_align(std::mem::size_of::<MeshDescription>() as u32);
        let position_offset = byteadressbuffer_align(indice_offset + indice_size);
        let normal_offset = byteadressbuffer_align(position_offset + position_size);
        let tangent_offset = byteadressbuffer_align(normal_offset + normal_size);
        let texcoord_offset = byteadressbuffer_align(tangent_offset + tangent_size);
        let color_offset = byteadressbuffer_align(texcoord_offset + texcoord_size);
        let buf_size = byteadressbuffer_align(color_offset + color_size);

        let mut mesh_desc = MeshDescription::default();
        mesh_desc.set_attrib_mask(self.get_mesh_attrib_mask());
        mesh_desc.set_position_offset(position_offset.into());
        mesh_desc.set_normal_offset(normal_offset.into());
        mesh_desc.set_tangent_offset(tangent_offset.into());
        mesh_desc.set_tex_coord_offset(texcoord_offset.into());
        mesh_desc.set_index_offset(indice_offset.into());
        mesh_desc.set_index_count((indice_size / 2).into());
        mesh_desc.set_color_offset(color_offset.into());
        mesh_desc.set_bounding_sphere(self.bounding_sphere.into());

        macro_rules! write_slice {
            ($writer:expr, $data:expr, $offset:expr, $size:expr) => {
                assert_eq!($writer.len(), $offset as usize);
                let written = $writer.write_slice($data);
                assert_eq!(written, $size as usize);
            };
        }

        let mut writer = BinaryWriter::with_capacity(buf_size as usize);

        writer.write(&mesh_desc);

        writer.align(DEFAULT_ALIGNMENT);

        write_slice!(writer, &self.indices, indice_offset, indice_size);

        writer.align(DEFAULT_ALIGNMENT);

        write_slice!(writer, &self.positions, position_offset, position_size);

        writer.align(DEFAULT_ALIGNMENT);

        if let Some(normals) = &self.normals {
            write_slice!(
                writer,
                &lgn_math::pack_normals_r11g11b10(normals),
                normal_offset,
                normal_size
            );
        }

        writer.align(DEFAULT_ALIGNMENT);

        if let Some(tangents) = &self.tangents {
            write_slice!(
                writer,
                &lgn_math::pack_tangents_r11g10b10a1(tangents),
                tangent_offset,
                tangent_size
            );
        }

        writer.align(DEFAULT_ALIGNMENT);

        if let Some(tex_coords) = &self.tex_coords {
            write_slice!(writer, tex_coords, texcoord_offset, texcoord_size);
        }

        writer.align(DEFAULT_ALIGNMENT);

        if let Some(colors) = &self.colors {
            write_slice!(writer, colors, color_offset, color_size);
        }

        writer.align(DEFAULT_ALIGNMENT);

        (writer.take(), indice_offset)
    }

    pub fn num_vertices(&self) -> usize {
        self.positions.len()
    }

    pub fn num_indices(&self) -> usize {
        self.indices.len()
    }

    pub fn new_cube(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
             half_size, -half_size, -half_size,  1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
             half_size,  half_size, -half_size,  1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size,  half_size,  half_size,  1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size, -half_size,  half_size,  1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -x
            -half_size, -half_size,  half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            -half_size,  half_size, -half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            -half_size, -half_size, -half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            // +y
            -half_size,  half_size, -half_size,  0.0,  1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            -half_size,  half_size,  half_size,  0.0,  1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size,  half_size,  half_size,  0.0,  1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size,  half_size, -half_size,  0.0,  1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -y
            -half_size, -half_size,  half_size,  0.0, -1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            -half_size, -half_size, -half_size,  0.0, -1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size, -half_size, -half_size,  0.0, -1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size, -half_size,  half_size,  0.0, -1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            // +z
             half_size, -half_size,  half_size,  0.0,  0.0,  1.0, 0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
             half_size,  half_size,  half_size,  0.0,  0.0,  1.0, 0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            -half_size,  half_size,  half_size,  0.0,  0.0,  1.0, 0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            -half_size, -half_size,  half_size,  0.0,  0.0,  1.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -z
            -half_size, -half_size, -half_size,  0.0,  0.0, -1.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size, -half_size,  0.0,  0.0, -1.0, 0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size,  half_size, -half_size,  0.0,  0.0, -1.0, 0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size, -half_size, -half_size,  0.0,  0.0, -1.0, 0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
        ];

        let mut index_data: Vec<u16> = vec![];
        index_data.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        index_data.extend_from_slice(&[4, 5, 6, 4, 6, 7]);
        index_data.extend_from_slice(&[8, 9, 10, 8, 10, 11]);
        index_data.extend_from_slice(&[12, 13, 14, 12, 14, 15]);
        index_data.extend_from_slice(&[16, 17, 18, 16, 18, 19]);
        index_data.extend_from_slice(&[20, 21, 22, 20, 22, 23]);

        Self::from_vertex_data(&vertex_data, Some(index_data), MeshTopology::TriangleList)
    }

    pub fn new_pyramid(base_size: f32, height: f32) -> Self {
        let half_size = base_size / 2.0;
        let top_z = -half_size + height;

        let top_z_p = Vec3::new(0.0, 0.0, top_z);
        let edge1 = Vec3::new(half_size, -half_size, -half_size) - top_z_p;
        let edge2 = Vec3::new(half_size, half_size, -half_size) - top_z_p;
        let edge3 = Vec3::new(-half_size, half_size, -half_size) - top_z_p;
        let edge4 = Vec3::new(-half_size, -half_size, -half_size) - top_z_p;
        let normal1 = Vec3::cross(edge1, edge2).normalize();
        let normal2 = Vec3::cross(edge2, edge3).normalize();
        let normal3 = Vec3::cross(edge3, edge4).normalize();
        let normal4 = Vec3::cross(edge4, edge1).normalize();

        #[rustfmt::skip]
        let vertex_data = [
            // base
             half_size, -half_size, -half_size, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size,  half_size, -half_size, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size, -half_size, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size, -half_size, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // 1
             half_size,  half_size, -half_size, normal1.x, normal1.y, normal1.z, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size, -half_size, normal1.x, normal1.y, normal1.z, 0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
                   0.0,        0.0,      top_z, normal1.x, normal1.y, normal1.z, 0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            // 2
            -half_size,  half_size, -half_size, normal2.x, normal2.y, normal2.z, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size, -half_size, normal2.x, normal2.y, normal2.z, 0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
                   0.0,        0.0,      top_z, normal2.x, normal2.y, normal2.z, 0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            // 3
            -half_size, -half_size, -half_size, normal3.x, normal3.y, normal3.z, 0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size,  half_size, -half_size, normal3.x, normal3.y, normal3.z, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
                   0.0,        0.0,      top_z, normal3.x, normal3.y, normal3.z, 0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            // 4
             half_size, -half_size, -half_size, normal4.x, normal4.y, normal4.z, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size, -half_size, normal4.x, normal4.y, normal4.z, 0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
                   0.0,        0.0,      top_z, normal4.x, normal4.y, normal4.z, 0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
        ];

        let mut index_data: Vec<u16> = vec![];
        index_data.extend_from_slice(&[0, 2, 1, 0, 3, 2]);
        index_data.extend_from_slice(&[4, 6, 5]);
        index_data.extend_from_slice(&[7, 9, 8]);
        index_data.extend_from_slice(&[10, 12, 11]);
        index_data.extend_from_slice(&[13, 15, 14]);

        Self::from_vertex_data(&vertex_data, Some(index_data), MeshTopology::TriangleList)
    }

    pub fn new_plane(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            -half_size, -half_size, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0,
            -half_size,  half_size, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0,  1.0,
             half_size,  half_size, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,  1.0, -1.0,
             half_size, -half_size, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,  1.0,  1.0,
        ];

        let mut index_data: Vec<u16> = vec![];
        index_data.extend_from_slice(&[0, 2, 1, 0, 3, 2]);

        Self::from_vertex_data(&vertex_data, Some(index_data), MeshTopology::TriangleList)
    }

    fn new_cylinder_inner(radius: f32, length: f32, steps: u32) -> (Vec<f32>, Vec<u16>) {
        let mut vertex_data = Vec::<f32>::new();

        let inc_angle = (2.0 * std::f32::consts::PI) / steps as f32;
        let mut cur_angle = 0.0f32;

        let base_point = Vec3::ZERO;
        let base_normal = DOWN_VECTOR;

        let top_point = Vec3::new(0.0, 0.0, length);
        let top_normal = UP_VECTOR;

        let mut current_index = 0u16;
        let mut index_data: Vec<u16> = vec![];
        for _i in 0..steps {
            let last_base_point = Vec3::new(cur_angle.cos(), cur_angle.sin(), 0.0).mul(radius);
            let last_top_point =
                Vec3::new(cur_angle.cos(), cur_angle.sin(), length / radius).mul(radius);

            cur_angle += inc_angle;

            let next_base_point = Vec3::new(cur_angle.cos(), cur_angle.sin(), 0.0).mul(radius);
            let next_top_point =
                Vec3::new(cur_angle.cos(), cur_angle.sin(), length / radius).mul(radius);

            // base
            add_vertex_data(&mut vertex_data, last_base_point, base_normal);
            add_vertex_data(&mut vertex_data, next_base_point, base_normal);
            add_vertex_data(&mut vertex_data, base_point, base_normal);
            index_data.extend_from_slice(&[current_index, current_index + 2, current_index + 1]);
            current_index += 3;

            // sides
            add_vertex_data(
                &mut vertex_data,
                last_base_point,
                last_base_point.normalize(),
            );
            add_vertex_data(
                &mut vertex_data,
                last_top_point,
                last_top_point.truncate().normalize().extend(0.0),
            );
            add_vertex_data(
                &mut vertex_data,
                next_base_point,
                next_base_point.normalize(),
            );
            add_vertex_data(
                &mut vertex_data,
                next_top_point,
                next_top_point.truncate().normalize().extend(0.0),
            );
            index_data.extend_from_slice(&[
                current_index,
                current_index + 2,
                current_index + 1,
                current_index + 2,
                current_index + 3,
                current_index + 1,
            ]);
            current_index += 4;

            // top
            add_vertex_data(&mut vertex_data, last_top_point, top_normal);
            add_vertex_data(&mut vertex_data, top_point, top_normal);
            add_vertex_data(&mut vertex_data, next_top_point, top_normal);
            index_data.extend_from_slice(&[current_index, current_index + 2, current_index + 1]);
            current_index += 3;
        }
        (vertex_data, index_data)
    }

    pub fn new_cylinder(radius: f32, length: f32, steps: u32) -> Self {
        let (vertex_data, index_data) = Self::new_cylinder_inner(radius, length, steps);
        Self::from_vertex_data(&vertex_data, Some(index_data), MeshTopology::TriangleList)
    }

    fn new_cone_inner(radius: f32, length: f32, steps: u32) -> (Vec<f32>, Vec<u16>) {
        let mut vertex_data = Vec::<f32>::new();

        let inc_angle = (2.0 * std::f32::consts::PI) / steps as f32;
        let mut cur_angle = 0.0f32;

        let base_point = Vec3::ZERO;
        let top_point = Vec3::new(0.0, 0.0, length);

        let base_normal = DOWN_VECTOR;

        let mut current_index = 0;
        let mut index_data: Vec<u16> = vec![];
        for _i in 0..steps {
            let last_base_point = Vec3::new(cur_angle.cos(), cur_angle.sin(), 0.0).mul(radius);

            cur_angle += inc_angle;

            let next_base_point = Vec3::new(cur_angle.cos(), cur_angle.sin(), 0.0).mul(radius);

            // base
            add_vertex_data(&mut vertex_data, last_base_point, base_normal);
            add_vertex_data(&mut vertex_data, next_base_point, base_normal);
            add_vertex_data(&mut vertex_data, base_point, base_normal);
            index_data.extend_from_slice(&[current_index, current_index + 2, current_index + 1]);
            current_index += 3;

            // side
            let last_base_normal = (top_point - last_base_point)
                .cross(Vec3::new(last_base_point.y, -last_base_point.x, 0.0))
                .normalize();
            let next_base_normal = (top_point - next_base_point)
                .cross(Vec3::new(next_base_point.y, -next_base_point.x, 0.0))
                .normalize();
            let top_normal = (last_base_normal + next_base_normal) / 2.0;
            add_vertex_data(&mut vertex_data, last_base_point, last_base_normal);
            add_vertex_data(&mut vertex_data, top_point, top_normal);
            add_vertex_data(&mut vertex_data, next_base_point, next_base_normal);
            index_data.extend_from_slice(&[current_index, current_index + 2, current_index + 1]);
            current_index += 3;
        }
        (vertex_data, index_data)
    }

    pub fn new_cone(radius: f32, length: f32, steps: u32) -> Self {
        let (vertex_data, index_data) = Self::new_cone_inner(radius, length, steps);
        Self::from_vertex_data(&vertex_data, Some(index_data), MeshTopology::TriangleList)
    }

    pub fn new_torus(
        circle_radius: f32,
        circle_steps: u32,
        torus_radius: f32,
        torus_steps: u32,
    ) -> Self {
        let mut vertex_data = Vec::<f32>::new();

        let inc_torus_angle = (2.0 * std::f32::consts::PI) / torus_steps as f32;
        let mut cur_torus_angle = 0.0f32;

        let mut current_index = 0u16;
        let mut index_data: Vec<u16> = vec![];

        for _i in 0..torus_steps {
            let last_torus_rot_normal = Mat4::from_axis_angle(Vec3::Z, cur_torus_angle);
            let last_torus_rot_point =
                last_torus_rot_normal * Mat4::from_translation(Vec3::new(torus_radius, 0.0, 0.0));

            cur_torus_angle += inc_torus_angle;

            let next_torus_rot_normal = Mat4::from_axis_angle(Vec3::Z, cur_torus_angle);
            let next_torus_rot_point =
                next_torus_rot_normal * Mat4::from_translation(Vec3::new(torus_radius, 0.0, 0.0));

            let inc_circle_angle = (2.0 * std::f32::consts::PI) / circle_steps as f32;
            let mut cur_circle_angle = 0.0f32;

            for _j in 0..circle_steps {
                let last_circle_point = Vec4::new(
                    cur_circle_angle.cos(),
                    0.0,
                    cur_circle_angle.sin(),
                    1.0 / circle_radius,
                )
                .mul(circle_radius);

                let last_circle_normal =
                    Vec4::new(cur_circle_angle.cos(), 0.0, cur_circle_angle.sin(), 0.0);

                cur_circle_angle += inc_circle_angle;

                let next_circle_point = Vec4::new(
                    cur_circle_angle.cos(),
                    0.0,
                    cur_circle_angle.sin(),
                    1.0 / circle_radius,
                )
                .mul(circle_radius);

                let next_circle_normal =
                    Vec4::new(cur_circle_angle.cos(), 0.0, cur_circle_angle.sin(), 0.0);

                let back_left_point = last_torus_rot_point.mul(last_circle_point);
                let back_left_normal = last_torus_rot_normal.mul(last_circle_normal);

                let back_right_point = last_torus_rot_point.mul(next_circle_point);
                let back_right_normal = last_torus_rot_normal.mul(next_circle_normal);

                let front_left_point = next_torus_rot_point.mul(last_circle_point);
                let front_left_normal = next_torus_rot_normal.mul(last_circle_normal);

                let front_right_point = next_torus_rot_point.mul(next_circle_point);
                let front_right_normal = next_torus_rot_normal.mul(next_circle_normal);

                add_vertex_data(
                    &mut vertex_data,
                    back_left_point.truncate(),
                    back_left_normal.truncate(),
                );
                add_vertex_data(
                    &mut vertex_data,
                    front_left_point.truncate(),
                    front_left_normal.truncate(),
                );
                add_vertex_data(
                    &mut vertex_data,
                    back_right_point.truncate(),
                    back_right_normal.truncate(),
                );
                add_vertex_data(
                    &mut vertex_data,
                    front_right_point.truncate(),
                    front_right_normal.truncate(),
                );

                index_data.extend_from_slice(&[
                    current_index,
                    current_index + 1,
                    current_index + 2,
                    current_index + 2,
                    current_index + 1,
                    current_index + 3,
                ]);
                current_index += 4;
            }
        }

        Self::from_vertex_data(&vertex_data, Some(index_data), MeshTopology::TriangleList)
    }

    pub fn new_wireframe_cube(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
             half_size, -half_size, -half_size, 1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size, 1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size, -half_size, 1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size,  half_size, -half_size, 1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size, -half_size,  half_size, 1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size,  half_size, -half_size, 1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // -x
            -half_size, -half_size, -half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size,  half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size,  half_size, -half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size,  half_size,  half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size, -half_size,  half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size,  half_size, -half_size, -1.0,  0.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // +y
             half_size,  half_size, -half_size, 0.0,  1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size,  half_size, -half_size, 0.0,  1.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size, 0.0,  1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size, 0.0,  1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -y
            -half_size, -half_size, -half_size, 0.0, -1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size, -half_size, 0.0, -1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size, 0.0, -1.0,  0.0, 0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size, 0.0, -1.0,  0.0, 0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
        ];

        Self::from_vertex_data(&vertex_data, None, MeshTopology::LineList)
    }

    pub fn new_ground_plane(num_squares: u32, num_sub_squares: u32, minor_spacing: f32) -> Self {
        let mut vertex_data = Vec::<f32>::new();

        let total_width = (num_squares * num_sub_squares) as f32 * minor_spacing;
        let half_width = total_width / 2.0;

        let mut x_inc = -half_width;
        let mut y_inc = -half_width;

        fn add_x_grid_line(
            vertex_data: &mut Vec<f32>,
            x_value: &mut f32,
            x_inc: f32,
            y_value: f32,
            grey_scale: f32,
        ) {
            vertex_data.append(&mut vec![
                *x_value, -y_value, 0.0, 0.0, 0.0, 1.0, grey_scale, grey_scale, grey_scale, 1.0,
                0.0, 1.0,
            ]);

            vertex_data.append(&mut vec![
                *x_value, y_value, 0.0, 0.0, 0.0, 1.0, grey_scale, grey_scale, grey_scale, 1.0,
                0.0, 1.0,
            ]);
            *x_value += x_inc;
        }

        for _x_outer in 0..num_squares {
            add_x_grid_line(
                &mut vertex_data,
                &mut x_inc,
                minor_spacing,
                half_width,
                0.05,
            );
            for _x_inner in 0..num_sub_squares - 1 {
                add_x_grid_line(&mut vertex_data, &mut x_inc, minor_spacing, half_width, 0.5);
            }
        }
        add_x_grid_line(
            &mut vertex_data,
            &mut x_inc,
            minor_spacing,
            half_width,
            0.05,
        );

        fn add_y_grid_line(
            vertex_data: &mut Vec<f32>,
            y_value: &mut f32,
            y_inc: f32,
            x_value: f32,
            grey_scale: f32,
        ) {
            vertex_data.append(&mut vec![
                -x_value, *y_value, 0.0, 0.0, 0.0, 1.0, grey_scale, grey_scale, grey_scale, 1.0,
                0.0, 1.0,
            ]);
            vertex_data.append(&mut vec![
                x_value, *y_value, 0.0, 0.0, 0.0, 1.0, grey_scale, grey_scale, grey_scale, 1.0,
                0.0, 1.0,
            ]);
            *y_value += y_inc;
        }

        for _y_outer in 0..num_squares {
            add_y_grid_line(
                &mut vertex_data,
                &mut y_inc,
                minor_spacing,
                half_width,
                0.05,
            );
            for _z_inner in 0..num_sub_squares - 1 {
                add_y_grid_line(&mut vertex_data, &mut y_inc, minor_spacing, half_width, 0.5);
            }
        }
        add_y_grid_line(
            &mut vertex_data,
            &mut y_inc,
            minor_spacing,
            half_width,
            0.05,
        );

        Self::from_vertex_data(&vertex_data, None, MeshTopology::LineList)
    }

    /// An arrow that points down with default rotation
    pub fn new_arrow() -> Self {
        let (mut arrow_vertex_data, mut arrow_index_data) = Self::new_cylinder_inner(0.01, 0.3, 10);
        let (mut cone_vertex_data, cone_index_data) = Self::new_cone_inner(0.025, 0.1, 10);
        for vertex_idx in 0..cone_vertex_data.len() / DEFAULT_MESH_VERTEX_SIZE {
            let array_idx = vertex_idx * DEFAULT_MESH_VERTEX_SIZE;
            // flip position and normals Z-coordinate
            cone_vertex_data[array_idx + 2] = -cone_vertex_data[array_idx + 2];
            cone_vertex_data[array_idx + 5] = -cone_vertex_data[array_idx + 5];
        }
        let mut cone_index_data = cone_index_data
            .iter()
            .map(|i| i + (arrow_vertex_data.len() / DEFAULT_MESH_VERTEX_SIZE) as u16)
            .collect();
        arrow_vertex_data.append(&mut cone_vertex_data);
        arrow_index_data.append(&mut cone_index_data);

        Self::from_vertex_data(
            &arrow_vertex_data,
            Some(arrow_index_data),
            MeshTopology::TriangleList,
        )
    }

    pub fn new_sphere(radius: f32, slices: u32, sails: u32) -> Self {
        let mut vertex_data = Vec::new();
        let slice_size = 2.0 * radius / slices as f32;
        let angle = 2.0 * std::f32::consts::PI / sails as f32;
        let v_delta = 1.0 / slices as f32;
        let u_delta = 1.0 / slices as f32;

        let mut current_index = 0u16;
        let mut index_data: Vec<u16> = vec![];

        for slice in 0..slices {
            let z0 = -radius + slice as f32 * slice_size;
            let z1 = -radius + (slice + 1) as f32 * slice_size;
            let v0 = slice as f32 * v_delta;
            let v1 = (slice + 1) as f32 * v_delta;
            for sail in 0..sails {
                let u0 = sail as f32 * u_delta;
                let u1 = (sail + 1) as f32 * u_delta;
                if slice == 0 {
                    let pole = Vec3::new(0.0, 0.0, z0);
                    let lr = (radius * radius - z1 * z1).sqrt();
                    let langle = angle * (sail as f32);
                    let p1 = Vec3::new(lr * langle.cos(), lr * langle.sin(), z1);
                    let n1 = p1.normalize();
                    let langle = angle * (sail + 1) as f32;
                    let p2 = Vec3::new(lr * langle.cos(), lr * langle.sin(), z1);
                    let n2 = p2.normalize();
                    vertex_data.append(&mut pole.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, -1.0]);
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u0, v0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u0, v1]);
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u1, v1]);

                    index_data.extend_from_slice(&[
                        current_index,
                        current_index + 1,
                        current_index + 2,
                    ]);
                    current_index += 3;
                } else if slice == slices - 1 {
                    let pole = Vec3::new(0.0, 0.0, z1);
                    let lr = (radius * radius - z0 * z0).sqrt();
                    let langle = angle * (sail as f32);
                    let p1 = Vec3::new(lr * langle.cos(), lr * langle.sin(), z0);
                    let n1 = p1.normalize();
                    let langle = angle * (sail + 1) as f32;
                    let p2 = Vec3::new(lr * langle.cos(), lr * langle.sin(), z0);
                    let n2 = p2.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u0, v0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u1, v0]);
                    vertex_data.append(&mut pole.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 1.0]);
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u0, v1]);

                    index_data.extend_from_slice(&[
                        current_index,
                        current_index + 1,
                        current_index + 2,
                    ]);
                    current_index += 3;
                } else {
                    let lr = (radius * radius - z0 * z0).sqrt();
                    let langle = angle * (sail as f32);
                    let p1 = Vec3::new(lr * langle.cos(), lr * langle.sin(), z0);
                    let n1 = p1.normalize();
                    let langle = angle * (sail + 1) as f32;
                    let p2 = Vec3::new(lr * langle.cos(), lr * langle.sin(), z0);
                    let n2 = p2.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u0, v0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u1, v0]);
                    let lr = (radius * radius - z1 * z1).sqrt();
                    let langle = angle * (sail as f32);
                    let p1 = Vec3::new(lr * langle.cos(), lr * langle.sin(), z1);
                    let n1 = p1.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u0, v1]);
                    let lr = (radius * radius - z1 * z1).sqrt();
                    let langle = angle * ((sail + 1) as f32);
                    let p1 = Vec3::new(lr * langle.cos(), lr * langle.sin(), z1);
                    let n1 = p1.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 1.0, u1, v1]);

                    index_data.extend_from_slice(&[
                        current_index,
                        current_index + 1,
                        current_index + 2,
                        current_index + 2,
                        current_index + 1,
                        current_index + 3,
                    ]);
                    current_index += 4;
                }
            }
        }

        Self::from_vertex_data(&vertex_data, Some(index_data), MeshTopology::TriangleList)
    }

    fn from_vertex_data(
        vertex_data: &[f32],
        indices: Option<Vec<u16>>,
        topology: MeshTopology,
    ) -> Self {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut colors = Vec::new();
        let mut tex_coords = Vec::new();
        assert_eq!(vertex_data.len() % DEFAULT_MESH_VERTEX_SIZE, 0);
        for i in 0..vertex_data.len() / DEFAULT_MESH_VERTEX_SIZE {
            let idx = i * DEFAULT_MESH_VERTEX_SIZE;
            positions.push(Vec3::new(
                vertex_data[idx],
                vertex_data[idx + 1],
                vertex_data[idx + 2],
            ));
            normals.push(Vec3::new(
                vertex_data[idx + 3],
                vertex_data[idx + 4],
                vertex_data[idx + 5],
            ));
            colors.push([
                (vertex_data[idx + 6] * 255.0) as u8,
                (vertex_data[idx + 7] * 255.0) as u8,
                (vertex_data[idx + 8] * 255.0) as u8,
                (vertex_data[idx + 9] * 255.0) as u8,
            ]);
            tex_coords.push(Vec2::new(vertex_data[idx + 10], vertex_data[idx + 11]));
        }

        let indices = indices.unwrap_or_else(|| {
            (0..positions.len())
                .map(|x| u16::try_from(x).unwrap())
                .collect::<Vec<u16>>()
        });

        let tangents = match topology {
            MeshTopology::LineList => {
                assert_eq!(indices.len() % 2, 0);
                None
            }
            MeshTopology::TriangleList => {
                assert_eq!(indices.len() % 3, 0);
                Some(
                    lgn_math::calculate_tangents(&positions, &tex_coords, &indices)
                        .iter()
                        .map(|v| v.extend(1.0))
                        .collect(),
                )
            }
        };

        let bounding_sphere = Self::calculate_bounding_sphere(&positions);

        Self {
            indices,
            positions,
            normals: Some(normals),
            tangents,
            tex_coords: Some(tex_coords),
            colors: Some(colors),
            // material_id: None,
            bounding_sphere,
            topology,
        }
    }

    pub fn calculate_tangents(&mut self) {
        assert!(self.tex_coords.is_some());
        self.tangents = Some(
            lgn_math::calculate_tangents(
                &self.positions,
                self.tex_coords.as_ref().unwrap(),
                &self.indices,
            )
            .iter()
            .map(|v| v.extend(-1.0))
            .collect(),
        );
    }
}
