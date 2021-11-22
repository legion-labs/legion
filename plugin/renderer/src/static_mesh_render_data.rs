use crate::renderer::Renderer;
use graphics_api::prelude::*;
use legion_math::Vec3;

pub struct StaticMeshRenderData {
    pub vertices: Vec<f32>,
    pub vertex_buffers: Vec<Buffer>,
}

impl StaticMeshRenderData {
    fn from_vertex_data(vertex_data: &[f32], renderer: &Renderer) -> Self {
        let mut vertex_buffers = Vec::with_capacity(renderer.num_render_frames as usize);
        for _ in 0..renderer.num_render_frames {
            let vertex_buffer = renderer
                .device_context()
                .create_buffer(&BufferDef::for_staging_vertex_buffer_data(vertex_data))
                .unwrap();
            vertex_buffer
                .copy_to_host_visible_buffer(vertex_data)
                .unwrap();
            vertex_buffers.push(vertex_buffer);
        }

        Self {
            vertices: vertex_data.to_vec(),
            vertex_buffers,
        }
    }

    pub fn num_vertices(&self) -> usize {
        self.vertices.len() / 6
    }

    pub fn new_cube(size: f32, renderer: &Renderer) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            // +x
            half_size, -half_size, -half_size, 1.0, 0.0, 0.0,
            half_size, half_size, -half_size, 1.0, 0.0, 0.0,
            half_size, half_size, half_size, 1.0, 0.0, 0.0,
            half_size, -half_size, -half_size, 1.0, 0.0, 0.0,
            half_size, half_size, half_size, 1.0, 0.0, 0.0,
            half_size, -half_size, half_size, 1.0, 0.0, 0.0,
            // -x
            -half_size, -half_size, -half_size, -1.0, 0.0, 0.0,
            -half_size, half_size, half_size, -1.0, 0.0, 0.0,
            -half_size, half_size, -half_size, -1.0, 0.0, 0.0,
            -half_size, -half_size, -half_size, -1.0, 0.0, 0.0,
            -half_size, -half_size, half_size, -1.0, 0.0, 0.0,
            -half_size, half_size, half_size, -1.0, 0.0, 0.0,
            // +y
            half_size, half_size, -half_size,   0.0, 1.0, 0.0,  
            -half_size, half_size, -half_size,  0.0, 1.0, 0.0,
            half_size, half_size, half_size,  0.0, 1.0, 0.0,
            half_size, half_size, half_size,  0.0, 1.0, 0.0,
            -half_size, half_size, -half_size,  0.0, 1.0, 0.0,
            -half_size, half_size, half_size,  0.0, 1.0, 0.0,
            // -y
            half_size, -half_size, -half_size,  0.0, -1.0, 0.0,
            half_size, -half_size, half_size, 0.0, -1.0, 0.0,
            -half_size, -half_size, -half_size, 0.0, -1.0, 0.0,
            half_size, -half_size, half_size, 0.0, -1.0, 0.0,
            -half_size, -half_size, half_size, 0.0, -1.0, 0.0,
            -half_size, -half_size, -half_size, 0.0, -1.0, 0.0,
            // +z
            half_size, -half_size, half_size, 0.0, 0.0, 1.0,
            half_size, half_size, half_size, 0.0, 0.0, 1.0,
            -half_size, -half_size, half_size, 0.0, 0.0, 1.0,
            -half_size, -half_size, half_size, 0.0, 0.0, 1.0,
            half_size, half_size, half_size, 0.0, 0.0, 1.0,
            -half_size, half_size, half_size, 0.0, 0.0, 1.0,
            // -z
            half_size, -half_size, -half_size, 0.0, 0.0, -1.0,
            -half_size, -half_size, -half_size, 0.0, 0.0, -1.0,
            half_size, half_size, -half_size, 0.0, 0.0, -1.0,
            -half_size, -half_size, -half_size, 0.0, 0.0, -1.0,
            -half_size, half_size, -half_size, 0.0, 0.0, -1.0,
            half_size, half_size, -half_size, 0.0, 0.0, -1.0,
        ];
        Self::from_vertex_data(&vertex_data, renderer)
    }

    pub fn new_pyramid(base_size: f32, height: f32, renderer: &Renderer) -> Self {
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
            half_size, -half_size, -half_size, 0.0, -1.0, 0.0,
            half_size, -half_size, half_size, 0.0, -1.0, 0.0,
            -half_size, -half_size, -half_size, 0.0, -1.0, 0.0,
            half_size, -half_size, half_size, 0.0, -1.0, 0.0,
            -half_size, -half_size, half_size, 0.0, -1.0, 0.0,
            -half_size, -half_size, -half_size, 0.0, -1.0, 0.0,
            // 1
            half_size, -half_size, -half_size, normal1.x, normal1.y, normal1.z,
            half_size, -half_size, half_size, normal1.x, normal1.y, normal1.z,
            0.0, top_y, 0.0, normal1.x, normal1.y, normal1.z,
            // 2
            half_size, -half_size, half_size, normal2.x, normal2.y, normal2.z,
            -half_size, -half_size, half_size, normal2.x, normal2.y, normal2.z,
            0.0, top_y, 0.0, normal2.x, normal2.y, normal2.z,
            // 3
            -half_size, -half_size, half_size, normal3.x, normal3.y, normal3.z,
            -half_size, -half_size, -half_size, normal3.x, normal3.y, normal3.z,
            0.0, top_y, 0.0, normal3.x, normal3.y, normal3.z,
            // 4
            -half_size, -half_size, -half_size, normal4.x, normal4.y, normal4.z,
            half_size, -half_size, -half_size, normal4.x, normal4.y, normal4.z,
            0.0, top_y, 0.0, normal4.x, normal4.y, normal4.z,
        ];
        Self::from_vertex_data(&vertex_data, renderer)
    }

    pub fn new_plane(size: f32, renderer: &Renderer) -> Self {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            -half_size, 0.0, -half_size, 0.0, 1.0, 0.0,
            -half_size, 0.0,  half_size, 0.0, 1.0, 0.0,
             half_size, 0.0, -half_size, 0.0, 1.0, 0.0,
             half_size, 0.0, -half_size, 0.0, 1.0, 0.0,
            -half_size, 0.0,  half_size, 0.0, 1.0, 0.0,
             half_size, 0.0,  half_size, 0.0, 1.0, 0.0,
        ];
        Self::from_vertex_data(&vertex_data, renderer)
    }
}
