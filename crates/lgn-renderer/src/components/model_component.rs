use std::ops::Mul;

use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::Component;
use lgn_graphics_data::runtime::MaterialReferenceType;
use lgn_math::{Mat4, Vec2, Vec3, Vec4};

use crate::{
    cgen::cgen_type::{MeshAttribMask, MeshDescription},
    resources::UniformGPUDataUpdater,
    DOWN_VECTOR, UP_VECTOR,
};

pub struct Mesh {
    pub positions: Option<Vec<Vec4>>,
    pub normals: Option<Vec<Vec4>>,
    pub tangents: Option<Vec<Vec4>>,
    pub tex_coords: Option<Vec<Vec2>>,
    pub indices: Option<Vec<u16>>,
    pub colors: Option<Vec<Vec4>>,

    pub material_id: Option<MaterialReferenceType>,
    pub bounding_sphere: Vec4,
}

impl Mesh {
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

    pub fn calculate_bounding_sphere(positions: &[Vec4]) -> Vec4 {
        let mut min_bound = Vec4::new(f32::MAX, f32::MAX, f32::MAX, 1.0);
        let mut max_bound = Vec4::new(f32::MIN, f32::MIN, f32::MIN, 1.0);

        for position in positions {
            min_bound = min_bound.min(*position);
            max_bound = max_bound.max(*position);
        }

        let delta = max_bound - min_bound;
        let mut mid_point = min_bound + delta * 0.5;

        let mut max_length: f32 = 0.0;
        for position in positions {
            let delta = *position - mid_point;
            let length = delta.abs().length();

            if length > max_length {
                max_length = length;
            }
        }
        mid_point.w = max_length;
        mid_point
    }

    pub fn make_gpu_update_job(
        &self,
        updater: &mut UniformGPUDataUpdater,
        offset: u32,
    ) -> (u32, u32, u32) {
        let mut mesh_desc = MeshDescription::default();
        mesh_desc.set_attrib_mask(self.get_mesh_attrib_mask());
        let mut offset = offset;
        let mut index_offset = 0;

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
            // Convert from byte offset to index offset, byte offset is only needed for uploading data
            index_offset = offset / 2;
            mesh_desc.set_index_offset(index_offset.into());
            mesh_desc.set_index_count((indices.len() as u32).into());
            updater.add_update_jobs(indices, u64::from(offset));
            offset += (std::mem::size_of::<u32>() * indices.len()) as u32;
        }
        if let Some(colors) = &self.colors {
            mesh_desc.set_color_offset(offset.into());
            updater.add_update_jobs(colors, u64::from(offset));
            offset += (std::mem::size_of::<Vec4>() * colors.len()) as u32;
        }
        mesh_desc.set_bounding_sphere(self.bounding_sphere.into());

        updater.add_update_jobs(&[mesh_desc], u64::from(offset));
        let mesh_info_offset = offset;
        offset += std::mem::size_of::<MeshDescription>() as u32;
        (offset, mesh_info_offset, index_offset)
    }

    pub fn num_vertices(&self) -> usize {
        if let Some(positions) = &self.positions {
            return positions.len();
        }
        0
    }

    pub fn num_indices(&self) -> usize {
        if let Some(indices) = &self.indices {
            return indices.len();
        }
        0
    }

    pub fn index_offset(&self) -> usize {
        if let Some(indices) = &self.indices {
            return indices.len();
        }
        0
    }

    pub fn new_cube(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
             half_size, -half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
             half_size,  half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size,  half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size, -half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -x
            -half_size, -half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            -half_size,  half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            -half_size, -half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            // +y
            -half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            -half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -y
            -half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            -half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            // +z
             half_size, -half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
            -half_size,  half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            -half_size, -half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -z
            -half_size, -half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
             half_size,  half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  0.0,
             half_size, -half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
        ];

        let mut index_data: Vec<u16> = vec![];
        index_data.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        index_data.extend_from_slice(&[4, 5, 6, 4, 6, 7]);
        index_data.extend_from_slice(&[8, 9, 10, 8, 10, 11]);
        index_data.extend_from_slice(&[12, 13, 14, 12, 14, 15]);
        index_data.extend_from_slice(&[16, 17, 18, 16, 18, 19]);
        index_data.extend_from_slice(&[20, 21, 22, 20, 22, 23]);

        Self::from_vertex_data(&vertex_data, Some(index_data))
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
            -half_size, -half_size,  half_size, 1.0,  0.0, -1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
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

        let mut index_data: Vec<u16> = vec![];
        index_data.extend_from_slice(&[0, 1, 2, 1, 3, 2]);
        index_data.extend_from_slice(&[4, 5, 6]);
        index_data.extend_from_slice(&[7, 8, 9]);
        index_data.extend_from_slice(&[10, 11, 12]);
        index_data.extend_from_slice(&[13, 14, 15]);

        Self::from_vertex_data(&vertex_data, Some(index_data))
    }

    pub fn new_plane(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            -half_size, 0.0, -half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0, -1.0, -1.0,
            -half_size, 0.0,  half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0, -1.0,  1.0,
             half_size, 0.0, -half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0,  1.0, -1.0,
             half_size, 0.0,  half_size, 1.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 0.0, 0.0,  1.0,  1.0,
        ];

        let mut index_data: Vec<u16> = vec![];
        index_data.extend_from_slice(&[0, 1, 2, 2, 1, 3]);

        Self::from_vertex_data(&vertex_data, Some(index_data))
    }

    pub fn new_cylinder(radius: f32, length: f32, steps: u32) -> Self {
        let mut vertex_data = Vec::<f32>::new();

        let inc_angle = (2.0 * std::f32::consts::PI) / steps as f32;
        let mut cur_angle = 0.0f32;

        let base_point = Vec3::ZERO;
        let base_normal = DOWN_VECTOR;

        let top_point = Vec3::new(0.0, length, 0.0);
        let top_normal = UP_VECTOR;

        let mut current_index = 0u16;
        let mut index_data: Vec<u16> = vec![];
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
            index_data.extend_from_slice(&[current_index, current_index + 1, current_index + 2]);
            current_index += 3;

            // sides
            add_vertex_data(&mut vertex_data, last_base_point, None);
            add_vertex_data(&mut vertex_data, last_top_point, None);
            add_vertex_data(&mut vertex_data, next_base_point, None);
            add_vertex_data(&mut vertex_data, next_top_point, None);
            index_data.extend_from_slice(&[
                current_index,
                current_index + 1,
                current_index + 2,
                current_index + 2,
                current_index + 1,
                current_index + 3,
            ]);
            current_index += 4;

            // top
            add_vertex_data(&mut vertex_data, last_top_point, Some(top_normal));
            add_vertex_data(&mut vertex_data, top_point, Some(top_normal));
            add_vertex_data(&mut vertex_data, next_top_point, Some(top_normal));
            index_data.extend_from_slice(&[current_index, current_index + 1, current_index + 2]);
            current_index += 3;
        }

        Self::from_vertex_data(&vertex_data, Some(index_data))
    }

    pub fn new_cone(radius: f32, length: f32, steps: u32, initial_index: u16) -> Self {
        let mut vertex_data = Vec::<f32>::new();

        let inc_angle = (2.0 * std::f32::consts::PI) / steps as f32;
        let mut cur_angle = 0.0f32;

        let base_point = Vec3::ZERO;
        let top_point = Vec3::new(0.0, length, 0.0);

        let base_normal = DOWN_VECTOR;

        let mut current_index = initial_index;
        let mut index_data: Vec<u16> = vec![];
        for _i in 0..steps {
            let last_base_point = Vec3::new(cur_angle.cos(), 0.0, cur_angle.sin()).mul(radius);

            cur_angle += inc_angle;

            let next_base_point = Vec3::new(cur_angle.cos(), 0.0, cur_angle.sin()).mul(radius);

            // base
            add_vertex_data(&mut vertex_data, last_base_point, Some(base_normal));
            add_vertex_data(&mut vertex_data, next_base_point, Some(base_normal));
            add_vertex_data(&mut vertex_data, base_point, Some(base_normal));
            index_data.extend_from_slice(&[current_index, current_index + 1, current_index + 2]);
            current_index += 3;

            // side
            add_vertex_data(&mut vertex_data, last_base_point, None);
            add_vertex_data(&mut vertex_data, top_point, None);
            add_vertex_data(&mut vertex_data, next_base_point, None);
            index_data.extend_from_slice(&[current_index, current_index + 1, current_index + 2]);
            current_index += 3;
        }

        Self::from_vertex_data(&vertex_data, Some(index_data))
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
                    front_right_point.truncate(),
                    Some(front_right_normal.truncate()),
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

        Self::from_vertex_data(&vertex_data, Some(index_data))
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
        Self::from_vertex_data(&vertex_data, None)
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

        Self::from_vertex_data(&vertex_data, None)
    }

    pub fn new_arrow() -> Self {
        let arrow = Self::new_cylinder(0.01, 0.3, 10);
        let initial_index = arrow.indices.as_ref().unwrap().len() as u16;
        let cone = Self::new_cone(0.025, 0.1, 10, initial_index);
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
        let mut indices = arrow.indices.unwrap();
        indices.append(&mut cone.indices.unwrap());

        let bounding_sphere = Self::calculate_bounding_sphere(&positions);

        Self {
            positions: Some(positions),
            normals: Some(normals),
            tangents: None,
            tex_coords: Some(tex_coords),
            colors: Some(colors),
            indices: Some(indices),

            material_id: None,
            bounding_sphere,
        }
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

                    index_data.extend_from_slice(&[
                        current_index,
                        current_index + 1,
                        current_index + 2,
                    ]);
                    current_index += 3;
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

                    index_data.extend_from_slice(&[
                        current_index,
                        current_index + 1,
                        current_index + 2,
                    ]);
                    current_index += 3;
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
                    let lr = (radius * radius - y1 * y1).sqrt();
                    let langle = angle * ((sail + 1) as f32);
                    let p1 = Vec3::new(lr * langle.cos(), y1, lr * langle.sin());
                    let n1 = p1.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, u1, v1]);

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

        Self::from_vertex_data(&vertex_data, Some(index_data))
    }

    fn from_vertex_data(vertex_data: &[f32], index_data: Option<Vec<u16>>) -> Self {
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
        let tangents = lgn_math::calculate_tangents(&positions, &tex_coords, &None);
        let bounding_sphere = Self::calculate_bounding_sphere(&positions);

        Self {
            positions: Some(positions),
            normals: Some(normals),
            tangents: Some(tangents),
            tex_coords: Some(tex_coords),
            indices: index_data,
            colors: Some(colors),

            material_id: None,
            bounding_sphere,
        }
    }

    pub fn calculate_tangents(&mut self) {
        assert!(self.positions.is_some());
        assert!(self.tex_coords.is_some());
        self.tangents = Some(lgn_math::calculate_tangents(
            self.positions.as_ref().unwrap(),
            self.tex_coords.as_ref().unwrap(),
            &self.indices,
        ));
    }
}

fn add_vertex_data(vertex_data: &mut Vec<f32>, pos: Vec3, normal_opt: Option<Vec3>) {
    let mut normal = Vec3::new(pos.x, 0.0, pos.z).normalize();
    if let Some(normal_opt) = normal_opt {
        normal = normal_opt;
    }
    vertex_data.append(&mut vec![
        pos.x, pos.y, pos.z, 1.0, normal.x, normal.y, normal.z, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]);
}

#[derive(Component)]
pub struct ModelComponent {
    pub model_id: Option<ResourceTypeAndId>,
    pub meshes: Vec<Mesh>,
}
