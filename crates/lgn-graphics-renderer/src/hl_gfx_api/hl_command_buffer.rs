use std::mem;

use lgn_graphics_api::{
    Buffer, BufferBarrier, BufferCopy, CmdBlitParams, CmdCopyBufferToTextureParams,
    CmdCopyTextureParams, ColorRenderTargetBinding, DepthStencilRenderTargetBinding,
    DescriptorSetHandle, DescriptorSetLayout, IndexBufferBinding, Pipeline, Texture,
    TextureBarrier, VertexBufferBinding,
};

use crate::resources::CommandBufferHandle;

pub struct HLCommandBuffer {
    cmd_buffer: CommandBufferHandle,
}

impl<'rc> HLCommandBuffer {
    pub fn new(mut cmd_buffer: CommandBufferHandle) -> Self {
        Self { cmd_buffer }
    }

    pub fn begin(&mut self) {
        self.cmd_buffer.begin();
    }

    pub fn end(&mut self) {
        self.cmd_buffer.end();
    }

    pub fn begin_render_pass(
        &mut self,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: &Option<DepthStencilRenderTargetBinding<'_>>,
    ) {
        self.cmd_buffer
            .cmd_begin_render_pass(color_targets, depth_target);
    }

    pub fn end_render_pass(&mut self) {
        self.cmd_buffer.cmd_end_render_pass();
    }

    pub fn with_label<F>(&mut self, label: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.cmd_buffer.begin_label(label);
        f(self);
        self.cmd_buffer.end_label();
    }

    pub fn bind_pipeline(&mut self, pipeline: &Pipeline) {
        self.cmd_buffer.cmd_bind_pipeline(pipeline);
    }

    pub fn bind_vertex_buffer(&mut self, first_binding: u32, binding: VertexBufferBinding) {
        self.bind_vertex_buffers(first_binding, std::slice::from_ref(&binding));
    }

    pub fn bind_vertex_buffers(&mut self, first_binding: u32, bindings: &[VertexBufferBinding]) {
        self.cmd_buffer
            .cmd_bind_vertex_buffers(first_binding, bindings);
    }

    pub fn bind_index_buffer(&mut self, binding: IndexBufferBinding) {
        self.cmd_buffer.cmd_bind_index_buffer(binding);
    }

    //
    // tmp? rely on a sort of cache. investigate!
    //

    pub fn bind_descriptor_set(
        &mut self,
        layout: &DescriptorSetLayout,
        handle: DescriptorSetHandle,
    ) {
        self.cmd_buffer
            .cmd_bind_descriptor_set_handle(layout, handle);
    }

    pub fn push_constant<T: Sized>(&mut self, constants: &T) {
        self.cmd_buffer.cmd_push_constant_typed(constants);
    }

    pub fn draw(&mut self, vertex_count: u32, first_vertex: u32) {
        self.cmd_buffer.cmd_draw(vertex_count, first_vertex);
    }

    pub fn draw_instanced(
        &mut self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        self.cmd_buffer.cmd_draw_instanced(
            vertex_count,
            first_vertex,
            instance_count,
            first_instance,
        );
    }

    pub fn draw_indirect(
        &mut self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        draw_count: u32,
        stride: u32,
    ) {
        self.cmd_buffer.cmd_draw_indirect(
            indirect_arg_buffer,
            indirect_arg_offset,
            draw_count,
            stride,
        );
    }

    pub fn draw_indirect_count(
        &mut self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        count_buffer: &Buffer,
        count_offset: u64,
        max_draw_count: u32,
        stride: u32,
    ) {
        self.cmd_buffer.cmd_draw_indirect_count(
            indirect_arg_buffer,
            indirect_arg_offset,
            count_buffer,
            count_offset,
            max_draw_count,
            stride,
        );
    }

    pub fn draw_indexed(&mut self, index_count: u32, first_index: u32, vertex_offset: i32) {
        self.cmd_buffer
            .cmd_draw_indexed(index_count, first_index, vertex_offset);
    }

    pub fn draw_indexed_instanced(
        &mut self,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) {
        self.cmd_buffer.cmd_draw_indexed_instanced(
            index_count,
            first_index,
            instance_count,
            first_instance,
            vertex_offset,
        );
    }

    pub fn draw_indexed_indirect(
        &mut self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        draw_count: u32,
        stride: u32,
    ) {
        self.cmd_buffer.cmd_draw_indexed_indirect(
            indirect_arg_buffer,
            indirect_arg_offset,
            draw_count,
            stride,
        );
    }

    pub fn draw_indexed_indirect_count(
        &mut self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        count_buffer: &Buffer,
        count_offset: u64,
        max_draw_count: u32,
        stride: u32,
    ) {
        self.cmd_buffer.cmd_draw_indexed_indirect_count(
            indirect_arg_buffer,
            indirect_arg_offset,
            count_buffer,
            count_offset,
            max_draw_count,
            stride,
        );
    }

    pub fn dispatch(&mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        self.cmd_buffer
            .cmd_dispatch(group_count_x, group_count_y, group_count_z);
    }

    pub fn dispatch_indirect(&mut self, buffer: &Buffer, offset: u64) {
        self.cmd_buffer.cmd_dispatch_indirect(buffer, offset);
    }

    pub fn resource_barrier(
        &mut self,
        buffer_barriers: &[BufferBarrier<'_>],
        texture_barriers: &[TextureBarrier<'_>],
    ) {
        self.cmd_buffer
            .cmd_resource_barrier(buffer_barriers, texture_barriers);
    }

    pub fn copy_buffer_to_buffer(
        &mut self,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        copy_data: &[BufferCopy],
    ) {
        self.cmd_buffer
            .cmd_copy_buffer_to_buffer(src_buffer, dst_buffer, copy_data);
    }

    pub fn copy_buffer_to_texture(
        &mut self,
        src_buffer: &Buffer,
        dst_texture: &Texture,
        params: &CmdCopyBufferToTextureParams,
    ) {
        self.cmd_buffer
            .cmd_copy_buffer_to_texture(src_buffer, dst_texture, params);
    }

    pub fn blit_texture(
        &mut self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdBlitParams,
    ) {
        self.cmd_buffer
            .cmd_blit_texture(src_texture, dst_texture, params);
    }

    pub fn copy_image(
        &mut self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdCopyTextureParams,
    ) {
        self.cmd_buffer
            .cmd_copy_image(src_texture, dst_texture, params);
    }

    pub fn fill_buffer(&mut self, dst_buffer: &Buffer, offset: u64, size: u64, data: u32) {
        self.cmd_buffer
            .cmd_fill_buffer(dst_buffer, offset, size, data);
    }

    pub fn finalize_(mut self) -> CommandBufferHandle {
        self.cmd_buffer.end();
        self.cmd_buffer.transfer()
    }
}

impl Drop for HLCommandBuffer {
    fn drop(&mut self) {
        if self.cmd_buffer.is_valid() {
            self.cmd_buffer.end();
        }
    }
}
