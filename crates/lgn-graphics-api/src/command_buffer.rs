use std::sync::atomic::{AtomicBool, Ordering};

use crate::backends::BackendCommandBuffer;
use crate::{
    Buffer, BufferCopy, CommandPool, DescriptorSetHandle, DescriptorSetLayout, Pipeline, Texture,
};
use crate::{
    BufferBarrier, CmdBlitParams, CmdCopyBufferToTextureParams, CmdCopyTextureParams,
    ColorRenderTargetBinding, DepthStencilRenderTargetBinding, DeviceContext, IndexBufferBinding,
    QueueType, RootSignature, TextureBarrier, VertexBufferBinding,
};

/// Used to create a `CommandBuffer`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandBufferDef {
    /// Secondary command buffers are used to encode a single pass on multiple
    /// threads
    pub is_secondary: bool,
}

pub struct CommandBuffer {
    pub(crate) device_context: DeviceContext,
    pub(crate) queue_type: QueueType,
    pub(crate) queue_family_index: u32,
    has_active_renderpass: AtomicBool,
    cur_pipeline: Option<Pipeline>,
    pub(crate) backend_command_buffer: BackendCommandBuffer,
}

impl CommandBuffer {
    pub(crate) fn new(
        device_context: &DeviceContext,
        command_pool: &CommandPool,
        command_buffer_def: CommandBufferDef,
    ) -> Self {
        let backend_command_buffer = BackendCommandBuffer::new(command_pool, command_buffer_def);

        Self {
            device_context: device_context.clone(),
            queue_type: command_pool.queue_type(),
            queue_family_index: command_pool.queue_family_index(),
            has_active_renderpass: AtomicBool::new(false),
            cur_pipeline: None,
            backend_command_buffer,
        }
    }

    pub fn begin(&mut self) {
        self.cur_pipeline = None;
        self.backend_begin();
    }

    pub fn end(&mut self) {
        if self.has_active_renderpass.load(Ordering::Relaxed) {
            self.cmd_end_render_pass();
            self.has_active_renderpass.store(false, Ordering::Relaxed);
        }

        self.backend_end();
    }

    pub fn cmd_begin_render_pass(
        &mut self,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: &Option<DepthStencilRenderTargetBinding<'_>>,
    ) {
        assert!(
            !(color_targets.is_empty() && depth_target.is_none()),
            "No color or depth target supplied to cmd_begin_render_pass"
        );

        if self.has_active_renderpass.load(Ordering::Relaxed) {
            self.cmd_end_render_pass();
        }

        self.backend_cmd_begin_render_pass(color_targets, depth_target);

        self.has_active_renderpass.store(true, Ordering::Relaxed);
    }

    pub fn cmd_end_render_pass(&mut self) {
        self.backend_cmd_end_render_pass();
        self.has_active_renderpass.store(false, Ordering::Relaxed);
    }

    pub fn with_label<F>(&mut self, label: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.begin_label(label);
        f(self);
        self.end_label();
    }

    pub fn begin_label(&mut self, label: &str) {
        self.backend_begin_label(label);
    }

    pub fn end_label(&mut self) {
        self.backend_end_label();
    }

    pub fn cmd_set_viewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth_min: f32,
        depth_max: f32,
    ) {
        self.backend_cmd_set_viewport(x, y, width, height, depth_min, depth_max);
    }

    pub fn cmd_set_scissor(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.backend_cmd_set_scissor(x, y, width, height);
    }

    pub fn cmd_set_stencil_reference_value(&mut self, value: u32) {
        self.backend_cmd_set_stencil_reference_value(value);
    }

    pub fn cmd_bind_pipeline(&mut self, pipeline: &Pipeline) {
        self.cur_pipeline = Some(pipeline.clone());
        self.backend_cmd_bind_pipeline(pipeline);
    }

    pub fn cmd_bind_vertex_buffer(&mut self, first_binding: u32, binding: VertexBufferBinding) {
        self.cmd_bind_vertex_buffers(first_binding, std::slice::from_ref(&binding));
    }

    pub fn cmd_bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        bindings: &[VertexBufferBinding],
    ) {
        self.backend_cmd_bind_vertex_buffers(first_binding, bindings);
    }

    pub fn cmd_bind_index_buffer(&mut self, binding: IndexBufferBinding) {
        self.backend_cmd_bind_index_buffer(binding);
    }

    pub fn cmd_bind_descriptor_set_handle(
        &mut self,
        layout: &DescriptorSetLayout,
        handle: DescriptorSetHandle,
        // pipeline_type: PipelineType,
        // root_signature: &RootSignature,
        // set_index: u32,
        // descriptor_set_handle: DescriptorSetHandle,
    ) {
        assert!(self.cur_pipeline.is_some());

        let cur_pipeline = self.cur_pipeline.as_ref().unwrap().clone();
        let pipeline_type = cur_pipeline.pipeline_type();
        let root_signature = cur_pipeline.root_signature();
        let set_index = layout.frequency();

        assert_eq!(
            &root_signature.definition().descriptor_set_layouts[set_index as usize],
            layout
        );

        self.backend_cmd_bind_descriptor_set_handle(
            pipeline_type,
            root_signature,
            set_index,
            handle,
        );
    }

    pub fn cmd_push_constant_typed<T: Sized>(&mut self, constants: &T) {
        assert!(self.cur_pipeline.is_some());

        let cur_pipeline = self.cur_pipeline.as_ref().unwrap().clone();
        let root_signature = cur_pipeline.root_signature();

        assert!(&root_signature.definition().push_constant_def.is_some());
        assert_eq!(
            root_signature.definition().push_constant_def.unwrap().size as usize,
            std::mem::size_of::<T>()
        );

        let constants_size = std::mem::size_of::<T>();
        let constants_ptr = (constants as *const T).cast::<u8>();
        #[allow(unsafe_code)]
        let data = unsafe { &*std::ptr::slice_from_raw_parts(constants_ptr, constants_size) };
        self.cmd_push_constant(root_signature, data);
    }

    pub fn cmd_push_constant(&mut self, root_signature: &RootSignature, data: &[u8]) {
        self.backend_cmd_push_constant(root_signature, data);
    }

    pub fn cmd_draw(&mut self, vertex_count: u32, first_vertex: u32) {
        self.backend_cmd_draw(vertex_count, first_vertex);
    }

    pub fn cmd_draw_instanced(
        &mut self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        self.backend_cmd_draw_instanced(vertex_count, first_vertex, instance_count, first_instance);
    }

    pub fn cmd_draw_indirect(
        &mut self,
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
        &mut self,
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

    pub fn cmd_draw_indexed(&mut self, index_count: u32, first_index: u32, vertex_offset: i32) {
        self.backend_cmd_draw_indexed(index_count, first_index, vertex_offset);
    }

    pub fn cmd_draw_indexed_instanced(
        &mut self,
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
        &mut self,
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
        &mut self,
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

    pub fn cmd_dispatch(&mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        self.backend_cmd_dispatch(group_count_x, group_count_y, group_count_z);
    }

    pub fn cmd_dispatch_indirect(&mut self, buffer: &Buffer, offset: u64) {
        self.backend_cmd_dispatch_indirect(buffer, offset);
    }

    pub fn cmd_resource_barrier(
        &mut self,
        buffer_barriers: &[BufferBarrier<'_>],
        texture_barriers: &[TextureBarrier<'_>],
    ) {
        assert!(
            !self.has_active_renderpass.load(Ordering::Relaxed),
            "cmd_resource_barrier may not be called if inside render pass"
        );
        self.backend_cmd_resource_barrier(buffer_barriers, texture_barriers);
    }

    pub fn cmd_fill_buffer(&mut self, dst_buffer: &Buffer, offset: u64, size: u64, data: u32) {
        self.backend_cmd_fill_buffer(dst_buffer, offset, size, data);
    }

    pub fn cmd_copy_buffer_to_buffer(
        &mut self,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        copy_data: &[BufferCopy],
    ) {
        self.backend_cmd_copy_buffer_to_buffer(src_buffer, dst_buffer, copy_data);
    }

    pub fn cmd_copy_buffer_to_texture(
        &mut self,
        src_buffer: &Buffer,
        dst_texture: &Texture,
        params: &CmdCopyBufferToTextureParams,
    ) {
        self.backend_cmd_copy_buffer_to_texture(src_buffer, dst_texture, params);
    }

    pub fn cmd_blit_texture(
        &mut self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdBlitParams,
    ) {
        self.backend_cmd_blit_texture(src_texture, dst_texture, params);
    }

    pub fn cmd_copy_image(
        &mut self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdCopyTextureParams,
    ) {
        self.backend_cmd_copy_image(src_texture, dst_texture, params);
    }
}
