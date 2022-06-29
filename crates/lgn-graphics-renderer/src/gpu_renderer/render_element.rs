use lgn_graphics_api::CommandBuffer;

use crate::{
    features::GpuInstanceId,
    resources::{MaterialId, MeshTopology, RenderMesh},
};

#[derive(Clone, Copy)]
pub struct RenderElement {
    gpu_instance_id: GpuInstanceId,
    material_id: MaterialId,
    index_count: u32,
    index_offset: u32,
}

#[allow(dead_code)]
impl RenderElement {
    pub fn new(gpu_instance_id: GpuInstanceId, material_id: MaterialId, mesh: &RenderMesh) -> Self {
        assert_eq!(mesh.topology, MeshTopology::TriangleList);
        Self {
            gpu_instance_id,
            material_id,
            index_count: mesh.index_count.get(),
            index_offset: mesh.index_offset,
        }
    }

    pub fn gpu_instance_id(&self) -> GpuInstanceId {
        self.gpu_instance_id
    }

    pub fn material_id(&self) -> MaterialId {
        self.material_id
    }

    pub fn draw(&self, cmd_buffer: &mut CommandBuffer) {
        cmd_buffer.cmd_draw_indexed_instanced(
            self.index_count,
            self.index_offset,
            1,
            self.gpu_instance_id.index(),
            0,
        );
    }
}
