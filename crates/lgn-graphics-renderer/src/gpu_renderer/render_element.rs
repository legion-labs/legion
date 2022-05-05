use crate::{
    hl_gfx_api::HLCommandBuffer,
    resources::{MaterialId, MeshMetaData},
};

use super::GpuInstanceId;

#[derive(Clone, Copy)]
pub struct RenderElement {
    gpu_instance_id: GpuInstanceId,
    material_id: MaterialId,
    vertex_count: u32,
    index_count: u32,
    index_offset: u32,
}

impl RenderElement {
    pub fn new(
        gpu_instance_id: GpuInstanceId,
        material_id: MaterialId,
        mesh: &MeshMetaData,
    ) -> Self {
        Self {
            gpu_instance_id,
            material_id,
            vertex_count: mesh.vertex_count,
            index_count: mesh.index_count,
            index_offset: mesh.index_offset,
        }
    }

    pub fn gpu_instance_id(&self) -> GpuInstanceId {
        self.gpu_instance_id
    }

    pub fn material_id(&self) -> MaterialId {
        self.material_id
    }

    pub fn draw(&self, cmd_buffer: &mut HLCommandBuffer) {
        if self.index_count != 0 {
            cmd_buffer.draw_indexed_instanced(
                self.index_count,
                self.index_offset,
                1,
                self.gpu_instance_id.index(),
                0,
            );
        } else {
            cmd_buffer.draw_instanced(self.vertex_count, 0, 1, self.gpu_instance_id.index());
        }
    }
}
