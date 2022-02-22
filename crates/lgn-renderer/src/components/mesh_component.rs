use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::Component;
use lgn_graphics_data::runtime::{MaterialReferenceType, MeshReferenceType};
use lgn_math::{Vec2, Vec4};

use crate::{
    cgen::cgen_type::{MeshAttribMask, MeshDescription},
    resources::UniformGPUDataUpdater,
};

pub struct SubMesh {
    pub positions: Option<Vec<Vec4>>,
    pub normals: Option<Vec<Vec4>>,
    pub tangents: Option<Vec<Vec4>>,
    pub tex_coords: Option<Vec<Vec2>>,
    pub indices: Option<Vec<u32>>,
    pub colors: Option<Vec<Vec4>>,

    pub material_id: Option<MaterialReferenceType>,
}

impl SubMesh {
    pub fn get_mesh_attrib_mask(&self) -> MeshAttribMask {
        let mut format = MeshAttribMask::empty();
        if self.positions.is_some() {
            format |= MeshAttribMask::POSITION;
        }
        if self.normals.is_some() {
            format |= MeshAttribMask::NORMAL;
        }
        if self.tangents.is_some() {
            format |= MeshAttribMask::TANGENT;
        }
        if self.tex_coords.is_some() {
            format |= MeshAttribMask::TEX_COORD;
        }
        if self.indices.is_some() {
            format |= MeshAttribMask::INDEX;
        }
        if self.colors.is_some() {
            format |= MeshAttribMask::COLOR;
        }
        format
    }

    pub fn size_in_bytes(&self) -> u32 {
        let mut size = 0;

        if let Some(positions) = &self.positions {
            size += (std::mem::size_of::<Vec4>() * positions.len()) as u32;
        }
        if let Some(normals) = &self.normals {
            size += (std::mem::size_of::<Vec4>() * normals.len()) as u32;
        }
        if let Some(tangents) = &self.tangents {
            size += (std::mem::size_of::<Vec4>() * tangents.len()) as u32;
        }
        if let Some(tex_coords) = &self.tex_coords {
            size += (std::mem::size_of::<Vec2>() * tex_coords.len()) as u32;
        }
        if let Some(indices) = &self.indices {
            size += (std::mem::size_of::<u32>() * indices.len()) as u32;
        }
        if let Some(colors) = &self.colors {
            size += (std::mem::size_of::<Vec4>() * colors.len()) as u32;
        }
        size
    }

    pub fn make_gpu_update_job(
        &self,
        updater: &mut UniformGPUDataUpdater,
        offset: u32,
    ) -> (u32, u32) {
        let mut mesh_desc = MeshDescription::default();
        mesh_desc.set_attrib_mask(self.get_mesh_attrib_mask());
        let mut offset = offset;

        if let Some(positions) = &self.positions {
            mesh_desc.set_position_offset(offset.into());
            updater.add_update_jobs(positions, u64::from(offset));
            offset += (std::mem::size_of::<Vec4>() * positions.len()) as u32;
        }
        if let Some(normals) = &self.normals {
            mesh_desc.set_normal_offset(offset.into());
            updater.add_update_jobs(normals, u64::from(offset));
            offset += (std::mem::size_of::<Vec4>() * normals.len()) as u32;
        }
        if let Some(tangents) = &self.tangents {
            mesh_desc.set_tangent_offset(offset.into());
            updater.add_update_jobs(tangents, u64::from(offset));
            offset += (std::mem::size_of::<Vec4>() * tangents.len()) as u32;
        }
        if let Some(tex_coords) = &self.tex_coords {
            mesh_desc.set_tex_coord_offset(offset.into());
            updater.add_update_jobs(tex_coords, u64::from(offset));
            offset += (std::mem::size_of::<Vec2>() * tex_coords.len()) as u32;
        }
        if let Some(indices) = &self.indices {
            mesh_desc.set_index_offset(offset.into());
            updater.add_update_jobs(indices, u64::from(offset));
            offset += (std::mem::size_of::<u32>() * indices.len()) as u32;
        }
        if let Some(colors) = &self.colors {
            mesh_desc.set_color_offset(offset.into());
            updater.add_update_jobs(colors, u64::from(offset));
            offset += (std::mem::size_of::<Vec4>() * colors.len()) as u32;
        }
        updater.add_update_jobs(&[mesh_desc], u64::from(offset));
        let mesh_info_offset = offset;
        offset += std::mem::size_of::<MeshDescription>() as u32;
        (offset, mesh_info_offset)
    }

    pub fn num_vertices(&self) -> usize {
        if let Some(indices) = &self.indices {
            return indices.len();
        }
        if let Some(positions) = &self.positions {
            return positions.len();
        }
        unreachable!()
    }
}

#[derive(Component)]
pub struct MeshComponent {
    pub mesh_id: Option<ResourceTypeAndId>,
    pub submeshes: Vec<SubMesh>,
}

impl MeshComponent {
    fn size_in_bytes(&self) -> u32 {
        let mut size = 0;
        for submesh in &self.submeshes {
            size += submesh.size_in_bytes();
        }
        size
    }
}
