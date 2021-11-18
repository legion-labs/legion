use crate::renderer::Renderer;
use graphics_api::{prelude::*};

pub struct StaticMeshRenderData {
    pub vertices: Vec<f32>,
    pub vertex_buffers: Vec<Buffer>,
}

impl StaticMeshRenderData {
    fn from_vertex_data(vertex_data: Vec<f32>, renderer: &Renderer) -> StaticMeshRenderData {
        let mut vertex_buffers = Vec::with_capacity(renderer.num_render_frames as usize);
        for _ in 0..renderer.num_render_frames {
            let vertex_buffer = renderer.device_context()
                .create_buffer(&BufferDef::for_staging_vertex_buffer_data(&vertex_data))
                .unwrap();
            vertex_buffer
                .copy_to_host_visible_buffer(&vertex_data)
                .unwrap();
            vertex_buffers.push(vertex_buffer);
        }
        
        StaticMeshRenderData {
            vertices: vertex_data.to_vec(),
            vertex_buffers
        }
    }

    pub fn new_cube(size: f32, renderer: &Renderer) -> StaticMeshRenderData {
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
        StaticMeshRenderData::from_vertex_data(vertex_data.to_vec(), renderer)
    }

    pub fn new_pyramid(base_size: f32, height: f32, renderer: &Renderer) -> StaticMeshRenderData {
        let half_size = base_size / 2.0;
        let top_y = -half_size + height;
        #[rustfmt::skip]
        let vertex_data = [
            // base
            half_size, -half_size, -half_size,
            half_size, -half_size, half_size,
            -half_size, -half_size, -half_size,            
            half_size, -half_size, half_size,
            -half_size, -half_size, half_size,
            -half_size, -half_size, -half_size,
            // 1
            half_size, -half_size, -half_size,
            half_size, -half_size, half_size,
            0.0, top_y, 0.0,
            // 2
            -half_size, -half_size, half_size,
            -half_size, -half_size, half_size,
            0.0, top_y, 0.0,
            // 3
            -half_size, -half_size, half_size,
            half_size, -half_size, half_size,
            0.0, top_y, 0.0,
            // 4
            -half_size, -half_size, -half_size,
            half_size, -half_size, -half_size,
            0.0, top_y, 0.0,
        ];
        StaticMeshRenderData::from_vertex_data(vertex_data.to_vec(), renderer)
    }

    pub fn new_plane(size: f32, renderer: &Renderer) -> StaticMeshRenderData {
        let half_size = size / 2.0;
        #[rustfmt::skip]
        let vertex_data = [
            -half_size, 0.0, -half_size,
            -half_size, 0.0,  half_size,
             half_size, 0.0, -half_size,
             half_size, 0.0, -half_size,
            -half_size, 0.0,  half_size,
             half_size, 0.0,  half_size,
        ];
        StaticMeshRenderData::from_vertex_data(vertex_data.to_vec(), renderer)
    }
}
