use std::sync::atomic::{AtomicBool, Ordering};

use crate::backends::BackendCommandBuffer;
use crate::{
    Buffer, BufferCopy, CommandPool, DescriptorSetHandle, Pipeline, PipelineType, Texture,
};
use crate::{
    BufferBarrier, CmdBlitParams, CmdCopyBufferToTextureParams, CmdCopyTextureParams,
    ColorRenderTargetBinding, DepthStencilRenderTargetBinding, DeviceContext, GfxResult,
    IndexBufferBinding, QueueType, RootSignature, TextureBarrier, VertexBufferBinding,
};

/// Used to create a `CommandBuffer`
#[derive(Debug, Clone, PartialEq)]
pub struct CommandBufferDef {
    /// Secondary command buffers are used to encode a single pass on multiple
    /// threads
    pub is_secondary: bool,
}

pub(crate) struct CommandBufferInner {
    pub(crate) device_context: DeviceContext,
    pub(crate) queue_type: QueueType,
    pub(crate) queue_family_index: u32,
    has_active_renderpass: AtomicBool,
    pub(crate) backend_command_buffer: BackendCommandBuffer,
}

pub struct CommandBuffer {
    pub(crate) inner: Box<CommandBufferInner>,
}

impl CommandBuffer {
    pub(crate) fn new(
        device_context: &DeviceContext,
        command_pool: &CommandPool,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<Self> {
        let backend_command_buffer = BackendCommandBuffer::new(command_pool, command_buffer_def)?;

        Ok(Self {
            inner: Box::new(CommandBufferInner {
                device_context: device_context.clone(),
                queue_type: command_pool.queue_type(),
                queue_family_index: command_pool.queue_family_index(),
                has_active_renderpass: AtomicBool::new(false),
                backend_command_buffer,
            }),
        })
    }

    pub fn begin(&self) -> GfxResult<()> {
        self.backend_begin()
    }

    pub fn end(&self) -> GfxResult<()> {
        if self.inner.has_active_renderpass.load(Ordering::Relaxed) {
            self.cmd_end_render_pass();
            self.inner
                .has_active_renderpass
                .store(false, Ordering::Relaxed);
        }

        self.backend_end()?;

        Ok(())
    }

    pub fn cmd_begin_render_pass(
        &self,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: &Option<DepthStencilRenderTargetBinding<'_>>,
    ) -> GfxResult<()> {
        if self.inner.has_active_renderpass.load(Ordering::Relaxed) {
            self.cmd_end_render_pass();
        }

        if color_targets.is_empty() && depth_target.is_none() {
            return Err("No color or depth target supplied to cmd_begin_render_pass".into());
        }

        self.backend_cmd_begin_render_pass(color_targets, depth_target)?;

        self.inner
            .has_active_renderpass
            .store(true, Ordering::Relaxed);

        Ok(())
    }

    pub fn cmd_end_render_pass(&self) {
        self.backend_cmd_end_render_pass();
        self.inner
            .has_active_renderpass
            .store(false, Ordering::Relaxed);
    }

    pub fn cmd_set_viewport(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth_min: f32,
        depth_max: f32,
    ) {
        self.backend_cmd_set_viewport(x, y, width, height, depth_min, depth_max);
    }

    pub fn cmd_set_scissor(&self, x: u32, y: u32, width: u32, height: u32) {
        self.backend_cmd_set_scissor(x, y, width, height);
    }

    pub fn cmd_set_stencil_reference_value(&self, value: u32) {
        self.backend_cmd_set_stencil_reference_value(value);
    }

    pub fn cmd_bind_pipeline(&self, pipeline: &Pipeline) {
        self.backend_cmd_bind_pipeline(pipeline);
    }

    pub fn cmd_bind_vertex_buffers(
        &self,
        first_binding: u32,
        bindings: &[VertexBufferBinding<'_>],
    ) {
        self.backend_cmd_bind_vertex_buffers(first_binding, bindings);
    }

    pub fn cmd_bind_index_buffer(&self, binding: &IndexBufferBinding<'_>) {
        self.backend_cmd_bind_index_buffer(binding);
    }

    pub fn cmd_bind_descriptor_set_handle(
        &self,
        pipeline_type: PipelineType,
        root_signature: &RootSignature,
        set_index: u32,
        descriptor_set_handle: DescriptorSetHandle,
    ) {
        self.backend_cmd_bind_descriptor_set_handle(
            pipeline_type,
            root_signature,
            set_index,
            descriptor_set_handle,
        );
    }

    pub fn cmd_push_constant(&self, root_signature: &RootSignature, data: &[u8]) {
        self.backend_cmd_push_constant(root_signature, data);
    }

    pub fn cmd_draw(&self, vertex_count: u32, first_vertex: u32) {
        self.backend_cmd_draw(vertex_count, first_vertex);
    }

    pub fn cmd_draw_instanced(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        self.backend_cmd_draw_instanced(vertex_count, first_vertex, instance_count, first_instance);
    }

    pub fn cmd_draw_indirect(
        &self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        draw_count: u32,
        stride: u32,
    ) {
        self.backend_cmd_draw_indirect(
            indirect_arg_buffer,
            indirect_arg_offset,
            draw_count,
            stride,
        );
    }

    pub fn cmd_draw_indirect_count(
        &self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        count_buffer: &Buffer,
        count_offset: u64,
        max_draw_count: u32,
        stride: u32,
    ) {
        self.backend_cmd_draw_indirect_count(
            indirect_arg_buffer,
            indirect_arg_offset,
            count_buffer,
            count_offset,
            max_draw_count,
            stride,
        );
    }

    pub fn cmd_draw_indexed(&self, index_count: u32, first_index: u32, vertex_offset: i32) {
        self.backend_cmd_draw_indexed(index_count, first_index, vertex_offset);
    }

    pub fn cmd_draw_indexed_instanced(
        &self,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) {
        self.backend_cmd_draw_indexed_instanced(
            index_count,
            first_index,
            instance_count,
            first_instance,
            vertex_offset,
        );
    }

    pub fn cmd_draw_indexed_indirect(
        &self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        draw_count: u32,
        stride: u32,
    ) {
        self.backend_cmd_draw_indexed_indirect(
            indirect_arg_buffer,
            indirect_arg_offset,
            draw_count,
            stride,
        );
    }

    pub fn cmd_draw_indexed_indirect_count(
        &self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        count_buffer: &Buffer,
        count_offset: u64,
        max_draw_count: u32,
        stride: u32,
    ) {
        self.backend_cmd_draw_indexed_indirect_count(
            indirect_arg_buffer,
            indirect_arg_offset,
            count_buffer,
            count_offset,
            max_draw_count,
            stride,
        );
    }

    pub fn cmd_dispatch(&self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        self.backend_cmd_dispatch(group_count_x, group_count_y, group_count_z);
    }

    pub fn cmd_dispatch_indirect(&self, buffer: &Buffer, offset: u64) {
        self.backend_cmd_dispatch_indirect(buffer, offset);
    }

    pub fn cmd_resource_barrier(
        &self,
        buffer_barriers: &[BufferBarrier<'_>],
        texture_barriers: &[TextureBarrier<'_>],
    ) {
        assert!(
            !self.inner.has_active_renderpass.load(Ordering::Relaxed),
            "cmd_resource_barrier may not be called if inside render pass"
        );
        self.backedn_cmd_resource_barrier(buffer_barriers, texture_barriers);
    }

    pub fn cmd_fill_buffer(&self, dst_buffer: &Buffer, offset: u64, size: u64, data: u32) {
        self.backend_cmd_fill_buffer(dst_buffer, offset, size, data);
    }

    pub fn cmd_copy_buffer_to_buffer(
        &self,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        copy_data: &[BufferCopy],
    ) {
        self.backend_cmd_copy_buffer_to_buffer(src_buffer, dst_buffer, copy_data);
    }

    pub fn cmd_copy_buffer_to_texture(
        &self,
        src_buffer: &Buffer,
        dst_texture: &Texture,
        params: &CmdCopyBufferToTextureParams,
    ) {
        self.backend_cmd_copy_buffer_to_texture(src_buffer, dst_texture, params);
    }

    pub fn cmd_blit_texture(
        &self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdBlitParams,
    ) {
        self.backend_cmd_blit_texture(src_texture, dst_texture, params);
    }

    pub fn cmd_copy_image(
        &self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdCopyTextureParams,
    ) {
        self.backend_cmd_copy_image(src_texture, dst_texture, params);
    }
}
