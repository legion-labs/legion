use crate::{hl_gfx_api::HLCommandBuffer, resources::MeshManager};

#[derive(Clone)]
pub struct RenderElement {
    pub(super) gpu_instance_id: u32,
    vertex_count: u32,
    index_count: u32,
    index_offset: u32,
}

impl RenderElement {
    pub fn new(gpu_instance_id: u32, mesh_id: u32, mesh_manager: &MeshManager) -> Self {
        let mesh = mesh_manager.mesh_from_id(mesh_id);

        Self {
            gpu_instance_id,
            vertex_count: mesh.num_vertices() as u32,
            index_count: 0,
            index_offset: 0,
        }
    }

    pub fn draw(&self, cmd_buffer: &mut HLCommandBuffer<'_>) {
        if self.index_count != 0 {
            cmd_buffer.draw_indexed_instanced(
                self.index_count,
                self.index_offset,
                1,
                self.gpu_instance_id,
                0,
            );
        } else {
            cmd_buffer.draw_instanced(self.vertex_count, 0, 1, self.gpu_instance_id);
        }
    }
}
