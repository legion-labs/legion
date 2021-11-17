use legion_math::{EulerRot, Mat4, Quat, Vec3};

pub struct StaticMesh {
    pub vertices: Vec<f32>,
}

impl StaticMesh {
    pub fn new_cube(size: f32) -> StaticMesh {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
            half_size, -half_size, -half_size,
            half_size, half_size, -half_size,
            half_size, half_size, half_size,
            half_size, -half_size, -half_size,
            half_size, half_size, half_size,
            half_size, -half_size, half_size,
            // -x
            -half_size, -half_size, -half_size,
            -half_size, half_size, half_size,
            -half_size, half_size, -half_size,
            -half_size, -half_size, -half_size,
            -half_size, -half_size, half_size,
            -half_size, half_size, half_size,
            // +y
            half_size, half_size, -half_size,
            -half_size, half_size, -half_size,
            half_size, half_size, half_size,
            half_size, half_size, half_size,
            -half_size, half_size, -half_size,
            -half_size, half_size, half_size,
            // -y
            half_size, -half_size, -half_size,
            half_size, -half_size, half_size,
            -half_size, -half_size, -half_size,            
            half_size, -half_size, half_size,
            -half_size, -half_size, half_size,
            -half_size, -half_size, -half_size,
            // +z
            half_size, -half_size, half_size,
            half_size, half_size, half_size,
            -half_size, -half_size, half_size,
            -half_size, -half_size, half_size,
            half_size, half_size, half_size,
            -half_size, half_size, half_size,
            // -z
            half_size, -half_size, -half_size,
            -half_size, -half_size, -half_size,
            half_size, half_size, -half_size,
            -half_size, -half_size, -half_size,
            -half_size, half_size, -half_size,
            half_size, half_size, -half_size,
        ];
        StaticMesh {
            vertices: vertex_data.to_vec(),
        }
    }
}
