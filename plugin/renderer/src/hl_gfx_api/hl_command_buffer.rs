use lgn_graphics_api::{
    Buffer, BufferBarrier, BufferCopy, BufferSubAllocation, CmdBlitParams,
    CmdCopyBufferToTextureParams, CmdCopyTextureParams, ColorRenderTargetBinding,
    DepthStencilRenderTargetBinding, DescriptorSetHandle, IndexBufferBinding, IndexType, Pipeline,
    PipelineType, RootSignature, Texture, TextureBarrier, VertexBufferBinding,
    MAX_DESCRIPTOR_SET_LAYOUTS,
};
use lgn_graphics_cgen_runtime::PipelineDataProvider;

use crate::resources::{CommandBufferHandle, CommandBufferPoolHandle};

pub struct HLCommandBuffer<'rc> {
    cmd_buffer_pool: &'rc CommandBufferPoolHandle,
    cmd_buffer: CommandBufferHandle,
    cur_pipeline_state: Option<&'rc Pipeline>,
}

impl<'rc> HLCommandBuffer<'rc> {
    pub fn new(cmd_buffer_pool: &'rc CommandBufferPoolHandle) -> Self {
        let cmd_buffer = cmd_buffer_pool.acquire();
        cmd_buffer.begin().unwrap();
        Self {
            cmd_buffer_pool,
            cmd_buffer,
            cur_pipeline_state: None,
        }
    }

    pub fn begin_render_pass(
        &self,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: &Option<DepthStencilRenderTargetBinding<'_>>,
    ) {
        self.cmd_buffer
            .cmd_begin_render_pass(color_targets, depth_target)
            .unwrap();
    }

    pub fn end_render_pass(&self) {
        self.cmd_buffer.cmd_end_render_pass().unwrap();
    }

    pub fn bind_pipeline(&self, pipeline: &Pipeline) {
        self.cmd_buffer.cmd_bind_pipeline(pipeline).unwrap();
    }

    pub fn bind_vertex_buffers(&self, first_binding: u32, bindings: &[VertexBufferBinding<'_>]) {
        self.cmd_buffer
            .cmd_bind_vertex_buffers(first_binding, bindings)
            .unwrap();
    }

    pub fn bind_buffer_suballocation_as_vertex_buffer<AllocType>(
        &self,
        binding: u32,
        buffer_suballoc: &BufferSubAllocation<AllocType>,
    ) {
        self.bind_vertex_buffers(
            binding,
            &[VertexBufferBinding {
                buffer: &buffer_suballoc.buffer,
                byte_offset: buffer_suballoc.offset(),
            }],
        );
    }

    pub fn bind_index_buffer(&self, binding: &IndexBufferBinding<'_>) {
        self.cmd_buffer.cmd_bind_index_buffer(binding).unwrap();
    }

    pub fn bind_buffer_suballocation_as_index_buffer<AllocType>(
        &self,
        buffer_suballoc: &BufferSubAllocation<AllocType>,
        index_type: IndexType,
    ) {
        self.bind_index_buffer(&IndexBufferBinding {
            buffer: &buffer_suballoc.buffer,
            byte_offset: buffer_suballoc.offset(),
            index_type,
        });
    }

    pub fn bind_descriptor_set_handle(
        &self,
        pipeline_type: PipelineType,
        root_signature: &RootSignature,
        set_index: u32,
        descriptor_set_handle: DescriptorSetHandle,
    ) {
        self.cmd_buffer
            .cmd_bind_descriptor_set_handle(
                pipeline_type,
                root_signature,
                set_index,
                descriptor_set_handle,
            )
            .unwrap();
    }

    pub fn push_constants<T: Sized>(&self, root_signature: &RootSignature, constants: &T) {
        self.cmd_buffer
            .cmd_push_constants(root_signature, constants)
            .unwrap();
    }

    pub fn draw(&self, vertex_count: u32, first_vertex: u32) {
        self.cmd_buffer
            .cmd_draw(vertex_count, first_vertex)
            .unwrap();
    }

    pub fn draw_with_data(
        &self,
        pipeline_data: &impl PipelineDataProvider,
        vertex_count: u32,
        first_vertex: u32,
    ) {
        self.set_pipeline_data(pipeline_data);
        self.cmd_buffer
            .cmd_draw(vertex_count, first_vertex)
            .unwrap();
    }

    pub fn draw_instanced(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        self.cmd_buffer
            .cmd_draw_instanced(vertex_count, first_vertex, instance_count, first_instance)
            .unwrap();
    }

    pub fn draw_indexed(&self, index_count: u32, first_index: u32, vertex_offset: i32) {
        self.cmd_buffer
            .cmd_draw_indexed(index_count, first_index, vertex_offset)
            .unwrap();
    }

    pub fn draw_indexed_instanced(
        &self,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) {
        self.cmd_buffer
            .cmd_draw_indexed_instanced(
                index_count,
                first_index,
                instance_count,
                first_instance,
                vertex_offset,
            )
            .unwrap();
    }

    pub fn dispatch(&self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        self.cmd_buffer
            .cmd_dispatch(group_count_x, group_count_y, group_count_z)
            .unwrap();
    }

    pub fn resource_barrier(
        &self,
        buffer_barriers: &[BufferBarrier<'_>],
        texture_barriers: &[TextureBarrier<'_>],
    ) {
        self.cmd_buffer
            .cmd_resource_barrier(buffer_barriers, texture_barriers)
            .unwrap();
    }

    pub fn copy_buffer_to_buffer(
        &self,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        copy_data: &[BufferCopy],
    ) {
        self.cmd_buffer
            .cmd_copy_buffer_to_buffer(src_buffer, dst_buffer, copy_data);
    }

    pub fn copy_buffer_to_texture(
        &self,
        src_buffer: &Buffer,
        dst_texture: &Texture,
        params: &CmdCopyBufferToTextureParams,
    ) {
        self.cmd_buffer
            .cmd_copy_buffer_to_texture(src_buffer, dst_texture, params)
            .unwrap();
    }

    pub fn blit_texture(
        &self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdBlitParams,
    ) {
        self.cmd_buffer
            .cmd_blit_texture(src_texture, dst_texture, params)
            .unwrap();
    }

    pub fn copy_image(
        &self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdCopyTextureParams,
    ) {
        self.cmd_buffer
            .cmd_copy_image(src_texture, dst_texture, params)
            .unwrap();
    }

    pub fn fill_buffer(&self, dst_buffer: &Buffer, offset: u64, size: u64, data: u32) {
        self.cmd_buffer
            .cmd_fill_buffer(dst_buffer, offset, size, data);
    }

    pub fn finalize(mut self) -> CommandBufferHandle {
        self.cmd_buffer.end().unwrap();
        self.cmd_buffer.transfer()
    }

    fn set_pipeline_data(&self, pipeline_data: &impl PipelineDataProvider) {
        let pipeline = pipeline_data.pipeline();
        let pipeline_type = pipeline.pipeline_type();
        let root_signature = pipeline.root_signature();

        self.cmd_buffer.cmd_bind_pipeline(&pipeline);

        for i in 0..MAX_DESCRIPTOR_SET_LAYOUTS as u32 {
            let descriptor_set = pipeline_data.descriptor_set(i);
            if let Some(descriptor_set) = descriptor_set {
                self.cmd_buffer.cmd_bind_descriptor_set_handle(
                    pipeline_type,
                    &root_signature,
                    i,
                    descriptor_set,
                );
            }
        }
    }
}

impl<'rc> Drop for HLCommandBuffer<'rc> {
    fn drop(&mut self) {
        if self.cmd_buffer.is_valid() {
            self.cmd_buffer.end().unwrap();
            self.cmd_buffer_pool.release(self.cmd_buffer.transfer());
        }
    }
}
