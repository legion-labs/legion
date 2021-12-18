use lgn_math::Vec3;

pub struct StaticMeshRenderData {
    pub vertices: Vec<f32>,
}

impl StaticMeshRenderData {
    fn from_vertex_data(vertex_data: &[f32]) -> Self {
        Self {
            vertices: vertex_data.to_vec(),
        }
    }

    pub fn num_vertices(&self) -> usize {
        self.vertices.len() / 12
    }

    pub fn new_cube(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
             half_size, -half_size, -half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size, -half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size,  half_size,  half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size, -half_size, -half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size, -half_size,  half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // -x
            -half_size, -half_size, -half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size,  half_size,  half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size, -half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size, -half_size, -half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size,  half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size,  half_size,  half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // +y
             half_size,  half_size, -half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size,  half_size, -half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size,  half_size,  half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size, -half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size,  half_size,  half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // -y
             half_size, -half_size, -half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size,  half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size, -half_size, -half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            // +z
             half_size, -half_size,  half_size,  0.0,  0.0,  1.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size,  half_size,  half_size,  0.0,  0.0,  1.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size,  0.0,  0.0,  1.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size,  half_size,  0.0,  0.0,  1.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size,  0.0,  0.0,  1.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size,  0.0,  0.0,  1.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // -z
             half_size, -half_size, -half_size,  0.0,  0.0, -1.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size, -half_size, -half_size,  0.0,  0.0, -1.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size, -half_size,  0.0,  0.0, -1.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size,  0.0,  0.0, -1.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size,  half_size, -half_size,  0.0,  0.0, -1.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
             half_size,  half_size, -half_size,  0.0,  0.0, -1.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
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
             half_size, -half_size, -half_size, 0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size,  half_size, 0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size, 0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size, 0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size, 0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size, -half_size, -half_size, 0.0, -1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            // 1
             half_size, -half_size, -half_size, normal1.x, normal1.y, normal1.z,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size, normal1.x, normal1.y, normal1.z,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
                   0.0,       top_y,       0.0, normal1.x, normal1.y, normal1.z,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            // 2
             half_size, -half_size,  half_size, normal2.x, normal2.y, normal2.z,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size,  half_size, normal2.x, normal2.y, normal2.z,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
                   0.0,      top_y,        0.0, normal2.x, normal2.y, normal2.z,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
            // 3
            -half_size, -half_size,  half_size, normal3.x, normal3.y, normal3.z,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size, -half_size, -half_size, normal3.x, normal3.y, normal3.z,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
                   0.0,      top_y,        0.0, normal3.x, normal3.y, normal3.z,  0.0, 0.0, 0.0, 1.0,  1.0,  0.0,
            // 4
            -half_size, -half_size, -half_size, normal4.x, normal4.y, normal4.z,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size, -half_size, normal4.x, normal4.y, normal4.z,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
                   0.0,       top_y,       0.0, normal4.x, normal4.y, normal4.z,  0.0, 0.0, 0.0, 1.0,  0.0,  1.0,
        ];
        Self::from_vertex_data(&vertex_data)
    }

    pub fn new_plane(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            -half_size, 0.0, -half_size, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, 0.0,  half_size, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
             half_size, 0.0, -half_size, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, 0.0, -half_size, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size, 0.0,  half_size, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
             half_size, 0.0,  half_size, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
        ];
        Self::from_vertex_data(&vertex_data)
    }

    pub fn new_wireframe_cube(size: f32) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
             half_size, -half_size, -half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size, -half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size,  half_size, -half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size, -half_size,  half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
             half_size,  half_size,  half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
             half_size,  half_size, -half_size,  1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            // -x
            -half_size, -half_size, -half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size, -half_size,  half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size, -half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size,  half_size, -half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
            -half_size,  half_size,  half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size, -half_size,  half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0,  1.0,
            -half_size,  half_size, -half_size, -1.0,  0.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // +y
             half_size,  half_size, -half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
            -half_size,  half_size, -half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size,  half_size,  half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size,  half_size,  half_size,  0.0,  1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            // -y
            -half_size, -half_size, -half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0, -1.0,
             half_size, -half_size, -half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
            -half_size, -half_size,  half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0, -1.0, -1.0,
             half_size, -half_size,  half_size,  0.0, -1.0,  0.0,  0.0, 0.0, 0.0, 1.0,  1.0,  1.0,
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
                *x_value, 0.0, -z_value, 0.0, 1.0, 0.0, grey_scale, grey_scale, grey_scale, 1.0,
                0.0, 1.0,
            ]);

            vertex_data.append(&mut vec![
                *x_value, 0.0, z_value, 0.0, 1.0, 0.0, grey_scale, grey_scale, grey_scale, 1.0,
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

        fn add_z_grid_line(
            vertex_data: &mut Vec<f32>,
            z_value: &mut f32,
            z_inc: f32,
            x_value: f32,
            grey_scale: f32,
        ) {
            vertex_data.append(&mut vec![
                -x_value, 0.0, *z_value, 0.0, 1.0, 0.0, grey_scale, grey_scale, grey_scale, 1.0,
                0.0, 1.0,
            ]);
            vertex_data.append(&mut vec![
                x_value, 0.0, *z_value, 0.0, 1.0, 0.0, grey_scale, grey_scale, grey_scale, 1.0,
                0.0, 1.0,
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
}
