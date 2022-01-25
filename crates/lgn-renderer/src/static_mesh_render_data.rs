use std::ops::Mul;

use lgn_math::{Mat4, Vec3, Vec4};

pub struct StaticMeshRenderData {
    pub vertices: Vec<f32>,
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

impl StaticMeshRenderData {
    fn from_vertex_data(vertex_data: &[f32]) -> Self {
        Self {
            vertices: vertex_data.to_vec(),
        }
    }

    pub fn num_vertices(&self) -> usize {
        self.vertices.len() / 14
    }

    pub fn new_cube(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
             half_size, -half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size,  half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size, -half_size, -half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size, -half_size,  half_size, 1.0,  1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // -x
            -half_size, -half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size,  half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size, -half_size, -half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size,  half_size,  half_size, 1.0, -1.0,  0.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // +y
             half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size, -half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size,  half_size,  half_size, 1.0,  0.0,  1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // -y
             half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size, -half_size, -half_size, 1.0,  0.0, -1.0,  0.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            // +z
             half_size, -half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size, 1.0,  0.0,  0.0,  1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // -z
             half_size, -half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size, -half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size,  half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
             half_size,  half_size, -half_size, 1.0,  0.0,  0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
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
        let normal1 = Vec3::cross(edge2, edge1);
        let normal2 = Vec3::cross(edge3, edge2);
        let normal3 = Vec3::cross(edge4, edge3);
        let normal4 = Vec3::cross(edge1, edge4);

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
        let mut cylinder = Self::new_cylinder(0.01, 0.3, 10);
        let cone = Self::new_cone(0.025, 0.1, 10);
        let mut cone_vertices = cone
            .vertices
            .into_iter()
            .enumerate()
            .map(|(idx, v)| if idx % 14 == 1 { -v } else { v })
            .collect::<Vec<f32>>();
        cone_vertices.append(&mut cylinder.vertices);
        Self::from_vertex_data(&cone_vertices)
    }

    pub fn new_sphere(radius: f32, slices: u32, sails: u32) -> Self {
        let mut vertex_data = Vec::new();
        let slice_size = 2.0 * radius / slices as f32;
        let angle = 2.0 * std::f32::consts::PI / sails as f32;
        for slice in 0..slices {
            let y0 = -radius + slice as f32 * slice_size;
            let y1 = -radius + (slice + 1) as f32 * slice_size;
            for sail in 0..sails {
                #[allow(clippy::branches_sharing_code)]
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
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
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
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                    vertex_data.append(&mut pole.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut vec![0.0, 1.0, 0.0]);
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
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
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                    let lr = (radius * radius - y1 * y1).sqrt();
                    let langle = angle * (sail as f32);
                    let p1 = Vec3::new(lr * langle.cos(), y1, lr * langle.sin());
                    let n1 = p1.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                    vertex_data.append(&mut p2.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n2.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                    let lr = (radius * radius - y1 * y1).sqrt();
                    let langle = angle * ((sail + 1) as f32);
                    let p1 = Vec3::new(lr * langle.cos(), y1, lr * langle.sin());
                    let n1 = p1.normalize();
                    vertex_data.append(&mut p1.to_array().to_vec());
                    vertex_data.push(1.0);
                    vertex_data.append(&mut n1.to_array().to_vec());
                    vertex_data.append(&mut vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
                }
            }
        }

        Self::from_vertex_data(&vertex_data)
    }
}
