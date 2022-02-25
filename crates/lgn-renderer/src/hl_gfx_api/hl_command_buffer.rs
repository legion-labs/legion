use std::mem;

use lgn_graphics_api::{
    Buffer, BufferBarrier, BufferCopy, BufferSubAllocation, CmdBlitParams,
    CmdCopyBufferToTextureParams, CmdCopyTextureParams, ColorRenderTargetBinding,
    DepthStencilRenderTargetBinding, DescriptorSetHandle, IndexBufferBinding, IndexType, Pipeline,
    Texture, TextureBarrier, VertexBufferBinding,
};

use crate::resources::{CommandBufferHandle, CommandBufferPoolHandle};

pub struct HLCommandBuffer<'rc> {
    cmd_buffer_pool: &'rc CommandBufferPoolHandle,
    cmd_buffer: CommandBufferHandle,
    cur_pipeline: Option<Pipeline>, // tmp? find a way to make a local cache by keeping current info
}

impl<'rc> HLCommandBuffer<'rc> {
    pub fn new(cmd_buffer_pool: &'rc CommandBufferPoolHandle) -> Self {
        let cmd_buffer = cmd_buffer_pool.acquire();
        cmd_buffer.begin().unwrap();
        Self {
            cmd_buffer_pool,
            cmd_buffer,
            cur_pipeline: None,
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
        self.cmd_buffer.cmd_end_render_pass();
    }

    pub fn bind_pipeline(&mut self, pipeline: &Pipeline) {
        self.cur_pipeline = Some(pipeline.clone());
        self.cmd_buffer.cmd_bind_pipeline(pipeline);
    }

    pub fn bind_vertex_buffers(&self, first_binding: u32, bindings: &[VertexBufferBinding<'_>]) {
        self.cmd_buffer
            .cmd_bind_vertex_buffers(first_binding, bindings);
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
        self.cmd_buffer.cmd_bind_index_buffer(binding);
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

    //
    // tmp? rely on a sort of cache. investigate!
    //

    pub fn bind_descriptor_set_handle(&self, handle: DescriptorSetHandle) {
        assert!(self.cur_pipeline.is_some());

        let cur_pipeline = self.cur_pipeline.as_ref().unwrap();
        let pipeline_type = cur_pipeline.pipeline_type();
        let root_signature = cur_pipeline.root_signature();
        let set_index = handle.frequency;

        assert_eq!(
            root_signature.definition().descriptor_set_layouts[set_index as usize].uid(),
            handle.layout_uid
        );

        self.cmd_buffer.cmd_bind_descriptor_set_handle(
            pipeline_type,
            root_signature,
            set_index,
            handle,
        );
    }

    pub fn push_constant<T: Sized>(&self, constants: &T) {
        assert!(self.cur_pipeline.is_some());

        let cur_pipeline = self.cur_pipeline.as_ref().unwrap();
        let root_signature = cur_pipeline.root_signature();

        assert!(&root_signature.definition().push_constant_def.is_some());
        assert_eq!(
            root_signature.definition().push_constant_def.unwrap().size as usize,
            mem::size_of::<T>()
        );

        let constants_size = mem::size_of::<T>();
        let constants_ptr = (constants as *const T).cast::<u8>();
        #[allow(unsafe_code)]
        let data = unsafe { &*std::ptr::slice_from_raw_parts(constants_ptr, constants_size) };
        self.cmd_buffer.cmd_push_constant(root_signature, data);
    }

    pub fn draw(&self, vertex_count: u32, first_vertex: u32) {
        self.cmd_buffer.cmd_draw(vertex_count, first_vertex);
    }

    pub fn draw_instanced(
        &self,
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

    pub(crate) fn draw_indirect(
        &self,
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

    pub(crate) fn draw_indirect_count(
        &self,
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

    pub fn draw_indexed(&self, index_count: u32, first_index: u32, vertex_offset: i32) {
        self.cmd_buffer
            .cmd_draw_indexed(index_count, first_index, vertex_offset);
    }

    pub fn draw_indexed_instanced(
        &self,
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

    pub(crate) fn draw_indexed_indirect(
        &self,
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

    pub(crate) fn draw_indexed_indirect_count(
        &self,
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

    pub fn dispatch(&self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        self.cmd_buffer
            .cmd_dispatch(group_count_x, group_count_y, group_count_z);
    }

    pub fn resource_barrier(
        &self,
        buffer_barriers: &[BufferBarrier<'_>],
        texture_barriers: &[TextureBarrier<'_>],
    ) {
        self.cmd_buffer
            .cmd_resource_barrier(buffer_barriers, texture_barriers);
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
            .cmd_copy_buffer_to_texture(src_buffer, dst_texture, params);
    }

    pub fn blit_texture(
        &self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdBlitParams,
    ) {
        self.cmd_buffer
            .cmd_blit_texture(src_texture, dst_texture, params);
    }

    pub fn copy_image(
        &self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdCopyTextureParams,
    ) {
        self.cmd_buffer
            .cmd_copy_image(src_texture, dst_texture, params);
    }

    pub fn fill_buffer(&self, dst_buffer: &Buffer, offset: u64, size: u64, data: u32) {
        self.cmd_buffer
            .cmd_fill_buffer(dst_buffer, offset, size, data);
    }

    pub fn finalize(mut self) -> CommandBufferHandle {
        self.cmd_buffer.end().unwrap();
        self.cmd_buffer.transfer()
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
