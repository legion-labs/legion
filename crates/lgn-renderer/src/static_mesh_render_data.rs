use std::ops::Mul;

use lgn_math::{Mat4, Vec2, Vec3, Vec4};

use crate::cgen;
use crate::resources::UniformGPUDataUpdater;

pub struct StaticMeshRenderData {
    pub positions: Option<Vec<Vec4>>,
    pub normals: Option<Vec<Vec4>>,
    pub tangents: Option<Vec<Vec4>>,
    pub tex_coords: Option<Vec<Vec2>>,
    pub indices: Option<Vec<u32>>,
    pub colors: Option<Vec<Vec4>>,
}

// bitflags::bitflags! {
//     pub struct MeshFormat: u32 {
//         const POSITION = 0x0001;
//         const NORMAL = 0x0002;
//         const TANGENT = 0x0004;
//         const TEX_COORD = 0x0008;
//         const INDEX = 0x0010;
//         const COLOR = 0x0020;
//     }
// }

// impl Default for MeshFormat {
//     fn default() -> Self {
//         Self::empty()
//     }
// }

// #[derive(Default)]
// pub struct MeshInfo {
//     pub format: MeshFormat,
//     pub position_offset: u32,
//     pub normal_offset: u32,
//     pub tangent_offset: u32,
//     pub tex_coord_offset: u32,
//     pub index_offset: u32,
//     pub color_offset: u32,
// }

fn add_vertex_data(vertex_data: &mut Vec<f32>, pos: Vec3, normal_opt: Option<Vec3>) {
    let mut normal = Vec3::new(pos.x, 0.0, pos.z).normalize();
    if let Some(normal_opt) = normal_opt {
        normal = normal_opt;
    }
    vertex_data.append(&mut vec![
        pos.x, pos.y, pos.z, 1.0, normal.x, normal.y, normal.z, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]);
}

impl StaticMeshRenderData {
    pub fn get_mesh_attrib_mask(&self) -> cgen::cgen_type::MeshAttribMask {
        let mut format = cgen::cgen_type::MeshAttribMask::empty();
        if self.positions.is_some() {
            format |= cgen::cgen_type::MeshAttribMask::POSITION;
        }
        if self.normals.is_some() {
            format |= cgen::cgen_type::MeshAttribMask::NORMAL;
        }
        if self.tangents.is_some() {
            format |= cgen::cgen_type::MeshAttribMask::TANGENT;
        }
        if self.tex_coords.is_some() {
            format |= cgen::cgen_type::MeshAttribMask::TEX_COORD;
        }
        if self.indices.is_some() {
            format |= cgen::cgen_type::MeshAttribMask::INDEX;
        }
        if self.colors.is_some() {
            format |= cgen::cgen_type::MeshAttribMask::COLOR;
        }
        format
    }

    pub fn make_gpu_update_job(
        &self,
        updater: &mut UniformGPUDataUpdater,
        offset: u32,
    ) -> (u32, cgen::cgen_type::MeshDescription) {
        let mut mesh_desc = cgen::cgen_type::MeshDescription::default();
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
        (offset, mesh_desc)
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

    fn from_vertex_data(vertex_data: &[f32]) -> Self {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut colors = Vec::new();
        let mut tex_coords = Vec::new();
        for i in 0..vertex_data.len() / 14 {
            let idx = i * 14;
            positions.push(Vec4::new(
                vertex_data[idx],
                vertex_data[idx + 1],
                vertex_data[idx + 2],
                vertex_data[idx + 3],
            ));
            normals.push(Vec4::new(
                vertex_data[idx + 4],
                vertex_data[idx + 5],
                vertex_data[idx + 6],
                vertex_data[idx + 7],
            ));
            colors.push(Vec4::new(
                vertex_data[idx + 8],
                vertex_data[idx + 9],
                vertex_data[idx + 10],
                vertex_data[idx + 11],
            ));
            tex_coords.push(Vec2::new(vertex_data[idx + 12], vertex_data[idx + 13]));
        }
        let tangents = calculate_tangents(&positions, &tex_coords, &None);
        Self {
            positions: Some(positions),
            normals: Some(normals),
            tangents: Some(tangents),
            tex_coords: Some(tex_coords),
            indices: None,
            colors: Some(colors),
        }
    }

    pub fn calculate_tangents(&mut self) {
        assert!(self.positions.is_some());
        assert!(self.tex_coords.is_some());
        self.tangents = Some(calculate_tangents(
            self.positions.as_ref().unwrap(),
            self.tex_coords.as_ref().unwrap(),
            &self.indices,
        ));
    }

    pub fn num_triangles(&self) -> usize {
        self.num_vertices() / 3
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

    pub fn new_cube(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
             half_size, -half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
             half_size,  half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size,  half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size, -half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
             half_size,  half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size, -half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -x
            -half_size, -half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            -half_size,  half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            -half_size, -half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            -half_size,  half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            // +y
             half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            -half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            -half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            // -y
             half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            -half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            // +z
             half_size, -half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            -half_size, -half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            -half_size,  half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            // -z
             half_size, -half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
             half_size,  half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            -half_size, -half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            -half_size,  half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size,  half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
        ];
        Self::from_vertex_data(&vertex_data)
    }

    pub fn new_pyramid(base_size: f32, height: f32) -> Self {
        let half_size = base_size / 2.0;
        let top_y = -half_size + height;

        let top_y_p = Vec3::new(0.0, top_y, 0.0);
        let edge1 = Vec3::new(half_size, -half_size, -half_size) - top_y_p;
        let edge2 = Vec3::new(half_size, -half_size, half_size) - top_y_p;
        let edge3 = Vec3::new(-half_size, -half_size, half_size) - top_y_p;
        let edge4 = Vec3::new(-half_size, -half_size, -half_size) - top_y_p;
        let normal1 = Vec3::cross(edge2, edge1).normalize();
        let normal2 = Vec3::cross(edge3, edge2).normalize();
        let normal3 = Vec3::cross(edge4, edge3).normalize();
        let normal4 = Vec3::cross(edge1, edge4).normalize();

        #[rustfmt::skip]
        let vertex_data = [
            // base
             half_size, -half_size, -half_size, 1.0,  0.0, -1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size,  half_size, 1.0,  0.0, -1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size, 1.0,  0.0, -1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size, 1.0,  0.0, -1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size, 1.0,  0.0, -1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size, -half_size, -half_size, 1.0,  0.0, -1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            // 1
             half_size, -half_size, -half_size, 1.0, normal1.x, normal1.y, normal1.z, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size, 1.0, normal1.x, normal1.y, normal1.z, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
                   0.0,       top_y,       0.0, 1.0, normal1.x, normal1.y, normal1.z, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            // 2
             half_size, -half_size,  half_size, 1.0, normal2.x, normal2.y, normal2.z, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size,  half_size, 1.0, normal2.x, normal2.y, normal2.z, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
                   0.0,      top_y,        0.0, 1.0, normal2.x, normal2.y, normal2.z, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            // 3
            -half_size, -half_size,  half_size, 1.0, normal3.x, normal3.y, normal3.z, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size, -half_size, -half_size, 1.0, normal3.x, normal3.y, normal3.z, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
                   0.0,      top_y,        0.0, 1.0, normal3.x, normal3.y, normal3.z, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            // 4
            -half_size, -half_size, -half_size, 1.0, normal4.x, normal4.y, normal4.z, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size, -half_size, 1.0, normal4.x, normal4.y, normal4.z, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
                   0.0,       top_y,       0.0, 1.0, normal4.x, normal4.y, normal4.z, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
        ];
        Self::from_vertex_data(&vertex_data)
    }

    pub fn new_plane(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            -half_size, 0.0, -half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0, -1.0, -1.0,
            -half_size, 0.0,  half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0, -1.0,  1.0,
             half_size, 0.0, -half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0,  1.0, -1.0,
             half_size, 0.0, -half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0,  1.0, -1.0,
            -half_size, 0.0,  half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0, -1.0,  1.0,
             half_size, 0.0,  half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0,  1.0,  1.0,
            -half_size, 0.0,  half_size, 0.0, 1.0, 0.0,
        ];
        Self::from_vertex_data(&vertex_data)
    }

    pub fn new_cylinder(radius: f32, length: f32, steps: u32) -> Self {
        let mut vertex_data = Vec::<f32>::new();

        let inc_angle = (2.0 * std::f32::consts::PI) / steps as f32;
        let mut cur_angle = 0.0f32;

        let base_point = Vec3::ZERO;
        let base_normal = Vec3::new(0.0, -1.0, 0.0);

        let top_point = Vec3::new(0.0, length, 0.0);
        let top_normal = Vec3::new(0.0, 1.0, 0.0);

        for _i in 0..steps {
            let last_base_point = Vec3::new(cur_angle.cos(), 0.0, cur_angle.sin()).mul(radius);
            let last_top_point =
                Vec3::new(cur_angle.cos(), length / radius, cur_angle.sin()).mul(radius);

            cur_angle += inc_angle;

            let next_base_point = Vec3::new(cur_angle.cos(), 0.0, cur_angle.sin()).mul(radius);
            let next_top_point =
                Vec3::new(cur_angle.cos(), length / radius, cur_angle.sin()).mul(radius);

            // base
            add_vertex_data(&mut vertex_data, last_base_point, Some(base_normal));
            add_vertex_data(&mut vertex_data, next_base_point, Some(base_normal));
            add_vertex_data(&mut vertex_data, base_point, Some(base_normal));

            // sides
            add_vertex_data(&mut vertex_data, last_base_point, None);
            add_vertex_data(&mut vertex_data, last_top_point, None);
            add_vertex_data(&mut vertex_data, next_base_point, None);

            add_vertex_data(&mut vertex_data, next_base_point, None);
            add_vertex_data(&mut vertex_data, last_top_point, None);
            add_vertex_data(&mut vertex_data, next_top_point, None);

            // top
            add_vertex_data(&mut vertex_data, last_top_point, Some(top_normal));
            add_vertex_data(&mut vertex_data, top_point, Some(top_normal));
            add_vertex_data(&mut vertex_data, next_top_point, Some(top_normal));
        }

        Self::from_vertex_data(&vertex_data)
    }

    pub fn new_cone(radius: f32, length: f32, steps: u32) -> Self {
        let mut vertex_data = Vec::<f32>::new();

        let inc_angle = (2.0 * std::f32::consts::PI) / steps as f32;
        let mut cur_angle = 0.0f32;

        let base_point = Vec3::ZERO;
        let top_point = Vec3::new(0.0, length, 0.0);

        let base_normal = Vec3::new(0.0, -1.0, 0.0);

        for _i in 0..steps {
            let last_base_point = Vec3::new(cur_angle.cos(), 0.0, cur_angle.sin()).mul(radius);

            cur_angle += inc_angle;

            let next_base_point = Vec3::new(cur_angle.cos(), 0.0, cur_angle.sin()).mul(radius);

            // base
            add_vertex_data(&mut vertex_data, last_base_point, Some(base_normal));
            add_vertex_data(&mut vertex_data, next_base_point, Some(base_normal));
            add_vertex_data(&mut vertex_data, base_point, Some(base_normal));

            // side
            add_vertex_data(&mut vertex_data, last_base_point, None);
            add_vertex_data(&mut vertex_data, top_point, None);
            add_vertex_data(&mut vertex_data, next_base_point, None);
        }

        Self::from_vertex_data(&vertex_data)
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

        for _i in 0..torus_steps {
            let last_torus_rot_normal =
                Mat4::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), cur_torus_angle);
            let last_torus_rot_point =
                last_torus_rot_normal * Mat4::from_translation(Vec3::new(torus_radius, 0.0, 0.0));

            cur_torus_angle += inc_torus_angle;

            let next_torus_rot_normal =
                Mat4::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), cur_torus_angle);
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
                    Some(back_left_normal.truncate()),
                );
                add_vertex_data(
                    &mut vertex_data,
                    front_left_point.truncate(),
                    Some(front_left_normal.truncate()),
                );
                add_vertex_data(
                    &mut vertex_data,
                    back_right_point.truncate(),
                    Some(back_right_normal.truncate()),
                );

                add_vertex_data(
                    &mut vertex_data,
                    back_right_point.truncate(),
                    Some(back_right_normal.truncate()),
                );
                add_vertex_data(
                    &mut vertex_data,
                    front_left_point.truncate(),
                    Some(front_left_normal.truncate()),
                );
                add_vertex_data(
                    &mut vertex_data,
                    front_right_point.truncate(),
                    Some(front_right_normal.truncate()),
                );
            }
        }

        Self::from_vertex_data(&vertex_data)
    }

    pub fn new_wireframe_cube(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
             half_size, -half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size,  half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size, -half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
             half_size,  half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size,  half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // -x
            -half_size, -half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size,  half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size,  half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size, -half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size,  half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // +y
             half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -y
            -half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
        ];
        Self::from_vertex_data(&vertex_data)
    }

    pub fn new_ground_plane(num_squares: u32, num_sub_squares: u32, minor_spacing: f32) -> Self {
        let mut vertex_data = Vec::<f32>::new();

        let total_width = (num_squares * num_sub_squares) as f32 * minor_spacing;
        let half_width = total_width / 2.0;

        let mut x_inc = -half_width;
        let mut z_inc = -half_width;

        fn add_x_grid_line(
            vertex_data: &mut Vec<f32>,
            x_value: &mut f32,
            x_inc: f32,
            z_value: f32,
            grey_scale: f32,
        ) {
            vertex_data.append(&mut vec![
                *x_value, 0.0, -z_value, 1.0, 0.0, 1.0, 0.0, 0.0, grey_scale, grey_scale,
                grey_scale, 1.0, 0.0, 1.0,
            ]);

            vertex_data.append(&mut vec![
                *x_value, 0.0, z_value, 1.0, 0.0, 1.0, 0.0, 0.0, grey_scale, grey_scale,
                grey_scale, 1.0, 0.0, 1.0,
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

        fn add_z_grid_line(
            vertex_data: &mut Vec<f32>,
            z_value: &mut f32,
            z_inc: f32,
            x_value: f32,
            grey_scale: f32,
        ) {
            vertex_data.append(&mut vec![
                -x_value, 0.0, *z_value, 1.0, 0.0, 1.0, 0.0, 0.0, grey_scale, grey_scale,
                grey_scale, 1.0, 0.0, 1.0,
            ]);
            vertex_data.append(&mut vec![
                x_value, 0.0, *z_value, 1.0, 0.0, 1.0, 0.0, 0.0, grey_scale, grey_scale,
                grey_scale, 1.0, 0.0, 1.0,
            ]);
            *z_value += z_inc;
        }

        for _z_outer in 0..num_squares {
            add_z_grid_line(
                &mut vertex_data,
                &mut z_inc,
                minor_spacing,
                half_width,
                0.05,
            );
            for _z_inner in 0..num_sub_squares - 1 {
                add_z_grid_line(&mut vertex_data, &mut z_inc, minor_spacing, half_width, 0.5);
            }
        }
        add_z_grid_line(
            &mut vertex_data,
            &mut z_inc,
            minor_spacing,
            half_width,
            0.05,
        );

        Self::from_vertex_data(&vertex_data)
    }

    pub fn new_arrow() -> Self {
        let arrow = Self::new_cylinder(0.01, 0.3, 10);
        let cone = Self::new_cone(0.025, 0.1, 10);
        let mut positions = arrow.positions.unwrap();
        positions.append(
            &mut cone
                .positions
                .unwrap()
                .into_iter()
                .map(|v| Vec4::new(v.x, -v.y, v.z, v.w))
                .collect(),
        );
        let mut normals = arrow.normals.unwrap();
        normals.append(
            &mut cone
                .normals
                .unwrap()
                .into_iter()
                .map(|v| Vec4::new(v.x, -v.y, v.z, v.w))
                .collect(),
        );
        let mut tex_coords = arrow.tex_coords.unwrap();
        tex_coords.append(&mut cone.tex_coords.unwrap());
        let mut colors = arrow.colors.unwrap();
        colors.append(&mut cone.colors.unwrap());

        Self {
            positions: Some(positions),
            normals: Some(normals),
            tangents: None,
            tex_coords: Some(tex_coords),
            colors: Some(colors),
            indices: None,
        }
    }

    pub fn new_sphere(radius: f32, slices: u32, sails: u32) -> Self {
        let mut vertex_data = Vec::new();
        let slice_size = 2.0 * radius / slices as f32;
        let angle = 2.0 * std::f32::consts::PI / sails as f32;
        let v_delta = 1.0 / slices as f32;
        let u_delta = 1.0 / slices as f32;
        for slice in 0..slices {
            let y0 = -radius + slice as f32 * slice_size;
            let y1 = -radius + (slice + 1) as f32 * slice_size;
            let v0 = slice as f32 * v_delta;
            let v1 = (slice + 1) as f32 * v_delta;
            for sail in 0..sails {
                let u0 = sail as f32 * u_delta;
                let u1 = (sail + 1) as f32 * u_delta;
                if slice == 0 {
                    let pole = Vec3::new(0.0, y0, 0.0);
                    let lr = (radius * radius - y1 * y1).sqrt();
                    let langle = angle * (sail as f32);
                    let p1 = Vec3::new(lr * langle.cos(), y1, lr * langle.sin());
                    let n1 = p1.normalize();
                    let langle = angle * (sail + 1) as f32;
                    let p2 = Vec3::new(lr * langle.cos(), y1, lr * langle.sin());
                    let n2 = p2.normalize();
                    vertex_data.append(&mut pole.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut vec![0.0, -1.0, 0.0]);
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u0, v0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u0, v1]);
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u1, v1]);
                } else if slice == slices - 1 {
                    let pole = Vec3::new(0.0, y1, 0.0);
                    let lr = (radius * radius - y0 * y0).sqrt();
                    let langle = angle * (sail as f32);
                    let p1 = Vec3::new(lr * langle.cos(), y0, lr * langle.sin());
                    let n1 = p1.normalize();
                    let langle = angle * (sail + 1) as f32;
                    let p2 = Vec3::new(lr * langle.cos(), y0, lr * langle.sin());
                    let n2 = p2.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u0, v0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u1, v0]);
                    vertex_data.append(&mut pole.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut vec![0.0, 1.0, 0.0]);
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u0, v1]);
                } else {
                    let lr = (radius * radius - y0 * y0).sqrt();
                    let langle = angle * (sail as f32);
                    let p1 = Vec3::new(lr * langle.cos(), y0, lr * langle.sin());
                    let n1 = p1.normalize();
                    let langle = angle * (sail + 1) as f32;
                    let p2 = Vec3::new(lr * langle.cos(), y0, lr * langle.sin());
                    let n2 = p2.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u0, v0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u1, v0]);
                    let lr = (radius * radius - y1 * y1).sqrt();
                    let langle = angle * (sail as f32);
                    let p1 = Vec3::new(lr * langle.cos(), y1, lr * langle.sin());
                    let n1 = p1.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u0, v1]);
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u0, v1]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u1, v0]);
                    let lr = (radius * radius - y1 * y1).sqrt();
                    let langle = angle * ((sail + 1) as f32);
                    let p1 = Vec3::new(lr * langle.cos(), y1, lr * langle.sin());
                    let n1 = p1.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u1, v1]);
                }
            }
        }

        Self::from_vertex_data(&vertex_data)
    }
}

#[allow(unsafe_code)]
fn calculate_tangents(
    positions: &[Vec4],
    tex_coords: &[Vec2],
    indices: &Option<Vec<u32>>,
) -> Vec<Vec4> {
    let length = positions.len();
    let mut tangents = Vec::with_capacity(length);
    //let mut bitangents = Vec::with_capacity(length);
    unsafe {
        tangents.set_len(length);
        //bitangents.set_len(length);
    }

    let num_triangles = if let Some(indices) = &indices {
        indices.len() / 3
    } else {
        length / 3
    };

    for i in 0..num_triangles {
        let idx0 = if let Some(indices) = &indices {
            indices[i * 3] as usize
        } else {
            i * 3
        };
        let idx1 = if let Some(indices) = &indices {
            indices[i * 3 + 1] as usize
        } else {
            i * 3 + 1
        };
        let idx2 = if let Some(indices) = &indices {
            indices[i * 3 + 2] as usize
        } else {
            i * 3 + 2
        };
        let v0 = positions[idx0].truncate();
        let v1 = positions[idx1].truncate();
        let v2 = positions[idx2].truncate();

        let uv0 = tex_coords[idx0];
        let uv1 = tex_coords[idx1];
        let uv2 = tex_coords[idx2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let f = delta_uv1.y * delta_uv2.x - delta_uv1.x * delta_uv2.y;
        //let b = (delta_uv2.x * edge1 - delta_uv1.x * edge2) / f;
        let t = (delta_uv1.y * edge2 - delta_uv2.y * edge1) / f;
        let t = t.extend(0.0);

        tangents[idx0] = t;
        tangents[idx1] = t;
        tangents[idx2] = t;

        //bitangents[idx0] = b;
        //bitangents[idx1] = b;
        //bitangents[idx2] = b;
    }

    tangents
}
