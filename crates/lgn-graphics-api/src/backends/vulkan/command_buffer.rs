use std::sync::Arc;

use lgn_tracing::trace;

use super::{internal, VkDebugReporter};
use crate::{
    BarrierQueueTransition, Buffer, BufferBarrier, BufferCopy, CmdBlitParams,
    CmdCopyBufferToTextureParams, CmdCopyTextureParams, ColorRenderTargetBinding, CommandBuffer,
    CommandBufferDef, CommandPool, DepthStencilRenderTargetBinding, DescriptorSetHandle,
    DeviceContext, GfxError, GfxResult, IndexBufferBinding, Pipeline, PipelineType, PlaneSlice,
    ResourceState, ResourceUsage, RootSignature, Texture, TextureBarrier, VertexBufferBinding,
};
pub(crate) struct VulkanCommandBuffer {
    vk_command_buffer: ash::vk::CommandBuffer,
    debug_reporter: Option<Arc<VkDebugReporter>>,
}

impl VulkanCommandBuffer {
    pub(crate) fn new(
        command_pool: &CommandPool,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<Self> {
        let vk_command_pool = command_pool.vk_command_pool();
        trace!("Creating command buffers from pool {:?}", vk_command_pool);
        let command_buffer_level = if command_buffer_def.is_secondary {
            ash::vk::CommandBufferLevel::SECONDARY
        } else {
            ash::vk::CommandBufferLevel::PRIMARY
        };

        let command_buffer_allocate_info = ash::vk::CommandBufferAllocateInfo::builder()
            .command_pool(vk_command_pool)
            .level(command_buffer_level)
            .command_buffer_count(1);

        let vk_command_buffer = unsafe {
            command_pool
                .device_context()
                .vk_device()
                .allocate_command_buffers(&command_buffer_allocate_info)
        }?[0];

        Ok(Self {
            vk_command_buffer,
            debug_reporter: command_pool
                .device_context()
                .debug_reporter()
                .as_ref()
                .cloned(),
        })
    }
}

impl CommandBuffer {
    pub(crate) fn vk_command_buffer(&mut self) -> ash::vk::CommandBuffer {
        self.inner.backend_command_buffer.vk_command_buffer
    }

    pub(crate) fn backend_begin(&mut self) -> GfxResult<()> {
        // TODO: check if it is not a ONE TIME SUBMIT
        let command_buffer_usage_flags = ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT;

        let begin_info =
            ash::vk::CommandBufferBeginInfo::builder().flags(command_buffer_usage_flags);

        unsafe {
            self.inner.device_context.vk_device().begin_command_buffer(
                self.inner.backend_command_buffer.vk_command_buffer,
                &*begin_info,
            )?;
        }

        Ok(())
    }

    pub(crate) fn backend_end(&mut self) -> GfxResult<()> {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .end_command_buffer(self.inner.backend_command_buffer.vk_command_buffer)?;
        }
        Ok(())
    }

    pub(crate) fn backend_cmd_begin_render_pass(
        &mut self,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: &Option<DepthStencilRenderTargetBinding<'_>>,
    ) -> GfxResult<()> {
        let barriers = {
            let mut barriers = Vec::with_capacity(color_targets.len() + 1);
            for color_target in color_targets {
                if color_target
                    .texture_view
                    .texture()
                    .take_is_undefined_layout()
                {
                    trace!(
                        "Transition RT {:?} from {:?} to {:?}",
                        color_target,
                        ResourceState::UNDEFINED,
                        ResourceState::RENDER_TARGET
                    );
                    barriers.push(TextureBarrier::state_transition(
                        color_target.texture_view.texture(),
                        ResourceState::UNDEFINED,
                        ResourceState::RENDER_TARGET,
                    ));
                }
            }

            if let Some(depth_target) = &depth_target {
                if depth_target
                    .texture_view
                    .texture()
                    .take_is_undefined_layout()
                {
                    trace!(
                        "Transition RT {:?} from {:?} to {:?}",
                        depth_target,
                        ResourceState::UNDEFINED,
                        ResourceState::DEPTH_WRITE
                    );
                    barriers.push(TextureBarrier::state_transition(
                        depth_target.texture_view.texture(),
                        ResourceState::UNDEFINED,
                        ResourceState::DEPTH_WRITE,
                    ));
                }
            }

            barriers
        };

        let extents = if let Some(first_color_rt) = color_targets.first() {
            let texture_def = first_color_rt.texture_view.texture().definition();
            let view_def = first_color_rt.texture_view.definition();
            let mut extents = texture_def.extents;
            extents.width >>= view_def.first_mip;
            extents.height >>= view_def.first_mip;
            extents
        } else if let Some(depth_rt) = &depth_target {
            let texture_def = depth_rt.texture_view.texture().definition();
            texture_def.extents
        } else {
            return Err(GfxError::String(
                "No render target in render pass color_targets or depth_target".to_string(),
            ));
        };

        let render_area = ash::vk::Rect2D {
            offset: ash::vk::Offset2D { x: 0, y: 0 },
            extent: ash::vk::Extent2D {
                width: extents.width,
                height: extents.height,
            },
        };

        if !barriers.is_empty() {
            self.backend_cmd_resource_barrier(&[], &barriers);
        }

        let mut color_attachments =
            vec![ash::vk::RenderingAttachmentInfo::default(); color_targets.len()];
        for (i, color_target) in color_targets.iter().enumerate() {
            color_attachments[i] = ash::vk::RenderingAttachmentInfo::builder()
                .image_view(color_target.texture_view.vk_image_view())
                .image_layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(color_target.load_op.into())
                .store_op(color_target.store_op.into())
                .clear_value(color_target.clear_value.into())
                .build();
        }

        let depth_attachment = depth_target.as_ref().map(|depth_target| {
            ash::vk::RenderingAttachmentInfo::builder()
                .image_view(depth_target.texture_view.vk_image_view())
                .image_layout(ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(depth_target.depth_load_op.into())
                .store_op(depth_target.depth_store_op.into())
                .clear_value(depth_target.clear_value.into())
                .build()
        });

        let stencil_attachment = depth_target.as_ref().map(|depth_target| {
            ash::vk::RenderingAttachmentInfo::builder()
                .image_view(depth_target.texture_view.vk_image_view())
                .image_layout(ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(depth_target.stencil_load_op.into())
                .store_op(depth_target.stencil_store_op.into())
                .clear_value(depth_target.clear_value.into())
                .build()
        });

        let render_info = ash::vk::RenderingInfo::builder()
            .render_area(render_area)
            .layer_count(1)
            .color_attachments(&color_attachments);

        let mut render_info = render_info.build();

        if depth_target.is_some() {
            render_info.p_depth_attachment = &depth_attachment.unwrap();
            render_info.p_stencil_attachment = &stencil_attachment.unwrap();
        };

        unsafe {
            self.inner.device_context.vk_device().cmd_begin_rendering(
                self.inner.backend_command_buffer.vk_command_buffer,
                &render_info,
            );
        }

        #[allow(clippy::cast_precision_loss)]
        self.cmd_set_viewport(
            0.0,
            0.0,
            extents.width as f32,
            extents.height as f32,
            0.0,
            1.0,
        );
        self.cmd_set_scissor(0, 0, extents.width, extents.height);

        Ok(())
    }

    pub(crate) fn backend_cmd_end_render_pass(&mut self) {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_end_rendering(self.inner.backend_command_buffer.vk_command_buffer);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn backend_cmd_set_viewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth_min: f32,
        depth_max: f32,
    ) {
        unsafe {
            // We invert the viewport by using negative height and setting y = y + height
            // This is supported in vulkan 1.1 or 1.0 with an extension
            self.inner.device_context.vk_device().cmd_set_viewport(
                self.inner.backend_command_buffer.vk_command_buffer,
                0,
                &[ash::vk::Viewport {
                    x,
                    y: y + height,
                    width,
                    height: height * -1.0,
                    min_depth: depth_min,
                    max_depth: depth_max,
                }],
            );
        }
    }

    pub(crate) fn backend_cmd_set_scissor(&mut self, x: u32, y: u32, width: u32, height: u32) {
        unsafe {
            self.inner.device_context.vk_device().cmd_set_scissor(
                self.inner.backend_command_buffer.vk_command_buffer,
                0,
                &[ash::vk::Rect2D {
                    offset: ash::vk::Offset2D {
                        x: x.try_into().unwrap(),
                        y: y.try_into().unwrap(),
                    },
                    extent: ash::vk::Extent2D { width, height },
                }],
            );
        }
    }

    pub(crate) fn backend_cmd_set_stencil_reference_value(&mut self, value: u32) {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_set_stencil_reference(
                    self.inner.backend_command_buffer.vk_command_buffer,
                    ash::vk::StencilFaceFlags::FRONT_AND_BACK,
                    value,
                );
        }
    }

    pub(crate) fn backend_cmd_bind_pipeline(&mut self, pipeline: &Pipeline) {
        //TODO: Add verification that the pipeline is compatible with the renderpass
        // created by the targets
        let pipeline_bind_point =
            super::internal::pipeline_type_pipeline_bind_point(pipeline.pipeline_type());

        unsafe {
            self.inner.device_context.vk_device().cmd_bind_pipeline(
                self.inner.backend_command_buffer.vk_command_buffer,
                pipeline_bind_point,
                pipeline.vk_pipeline(),
            );
        }
    }

    pub(crate) fn backend_cmd_bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        bindings: &[VertexBufferBinding<'_>],
    ) {
        let mut buffers = Vec::with_capacity(bindings.len());
        let mut offsets = Vec::with_capacity(bindings.len());
        for binding in bindings {
            buffers.push(binding.buffer.vk_buffer());
            offsets.push(binding.byte_offset);
        }

        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_bind_vertex_buffers(
                    self.inner.backend_command_buffer.vk_command_buffer,
                    first_binding,
                    &buffers,
                    &offsets,
                );
        }
    }

    pub(crate) fn backend_cmd_bind_index_buffer(&mut self, binding: &IndexBufferBinding<'_>) {
        unsafe {
            self.inner.device_context.vk_device().cmd_bind_index_buffer(
                self.inner.backend_command_buffer.vk_command_buffer,
                binding.buffer.vk_buffer(),
                binding.byte_offset,
                binding.index_type.into(),
            );
        }
    }

    pub(crate) fn backend_cmd_bind_descriptor_set_handle(
        &mut self,
        pipeline_type: PipelineType,
        root_signature: &RootSignature,
        set_index: u32,
        descriptor_set_handle: DescriptorSetHandle,
    ) {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_bind_descriptor_sets(
                    self.inner.backend_command_buffer.vk_command_buffer,
                    super::internal::pipeline_type_pipeline_bind_point(pipeline_type),
                    root_signature.vk_pipeline_layout(),
                    set_index,
                    &[descriptor_set_handle.backend_descriptor_set_handle],
                    &[],
                );
        }
    }

    pub(crate) fn backend_cmd_push_constant(
        &mut self,
        root_signature: &RootSignature,
        data: &[u8],
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_push_constants(
                self.inner.backend_command_buffer.vk_command_buffer,
                root_signature.vk_pipeline_layout(),
                ash::vk::ShaderStageFlags::ALL,
                0,
                data,
            );
        }
    }

    pub(crate) fn backend_cmd_draw(&mut self, vertex_count: u32, first_vertex: u32) {
        unsafe {
            self.inner.device_context.vk_device().cmd_draw(
                self.inner.backend_command_buffer.vk_command_buffer,
                vertex_count,
                1,
                first_vertex,
                0,
            );
        }
    }

    pub(crate) fn backend_cmd_draw_instanced(
        &mut self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_draw(
                self.inner.backend_command_buffer.vk_command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    pub(crate) fn backend_cmd_draw_indirect(
        &mut self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_draw_indirect(
                self.inner.backend_command_buffer.vk_command_buffer,
                indirect_arg_buffer.vk_buffer(),
                indirect_arg_offset,
                draw_count,
                stride,
            );
        }
    }

    pub(crate) fn backend_cmd_draw_indirect_count(
        &mut self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        count_buffer: &Buffer,
        count_offset: u64,
        max_draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_draw_indirect_count(
                    self.inner.backend_command_buffer.vk_command_buffer,
                    indirect_arg_buffer.vk_buffer(),
                    indirect_arg_offset,
                    count_buffer.vk_buffer(),
                    count_offset,
                    max_draw_count,
                    stride,
                );
        }
    }

    pub(crate) fn backend_cmd_draw_indexed(
        &mut self,
        index_count: u32,
        first_index: u32,
        vertex_offset: i32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_draw_indexed(
                self.inner.backend_command_buffer.vk_command_buffer,
                index_count,
                1,
                first_index,
                vertex_offset,
                0,
            );
        }
    }

    pub(crate) fn backend_cmd_draw_indexed_instanced(
        &mut self,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_draw_indexed(
                self.inner.backend_command_buffer.vk_command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    pub(crate) fn backend_cmd_draw_indexed_indirect(
        &mut self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_draw_indexed_indirect(
                    self.inner.backend_command_buffer.vk_command_buffer,
                    indirect_arg_buffer.vk_buffer(),
                    indirect_arg_offset,
                    draw_count,
                    stride,
                );
        }
    }

    pub(crate) fn backend_cmd_draw_indexed_indirect_count(
        &mut self,
        indirect_arg_buffer: &Buffer,
        indirect_arg_offset: u64,
        count_buffer: &Buffer,
        count_offset: u64,
        max_draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_draw_indexed_indirect_count(
                    self.inner.backend_command_buffer.vk_command_buffer,
                    indirect_arg_buffer.vk_buffer(),
                    indirect_arg_offset,
                    count_buffer.vk_buffer(),
                    count_offset,
                    max_draw_count,
                    stride,
                );
        }
    }

    pub(crate) fn backend_cmd_dispatch(
        &mut self,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_dispatch(
                self.inner.backend_command_buffer.vk_command_buffer,
                group_count_x,
                group_count_y,
                group_count_z,
            );
        }
    }

    pub(crate) fn backend_cmd_dispatch_indirect(&mut self, buffer: &Buffer, offset: u64) {
        unsafe {
            self.inner.device_context.vk_device().cmd_dispatch_indirect(
                self.inner.backend_command_buffer.vk_command_buffer,
                buffer.vk_buffer(),
                offset,
            );
        }
    }

    pub(crate) fn backend_cmd_resource_barrier(
        &mut self,
        buffer_barriers: &[BufferBarrier<'_>],
        texture_barriers: &[TextureBarrier<'_>],
    ) {
        let mut vk_image_barriers = Vec::with_capacity(texture_barriers.len());
        let mut vk_buffer_barriers = Vec::with_capacity(buffer_barriers.len());

        let mut src_access_flags = ash::vk::AccessFlags::empty();
        let mut dst_access_flags = ash::vk::AccessFlags::empty();

        for barrier in buffer_barriers {
            let mut vk_buffer_barrier = ash::vk::BufferMemoryBarrier::builder()
                .src_access_mask(super::internal::resource_state_to_access_flags(
                    barrier.src_state,
                ))
                .dst_access_mask(super::internal::resource_state_to_access_flags(
                    barrier.dst_state,
                ))
                .buffer(barrier.buffer.vk_buffer())
                .size(ash::vk::WHOLE_SIZE)
                .offset(0)
                .build();

            match &barrier.queue_transition {
                BarrierQueueTransition::ReleaseTo(dst_queue_type) => {
                    vk_buffer_barrier.src_queue_family_index = self.inner.queue_family_index;
                    vk_buffer_barrier.dst_queue_family_index =
                        super::internal::queue_type_to_family_index(
                            &self.inner.device_context,
                            *dst_queue_type,
                        );
                }
                BarrierQueueTransition::AcquireFrom(src_queue_type) => {
                    vk_buffer_barrier.src_queue_family_index =
                        super::internal::queue_type_to_family_index(
                            &self.inner.device_context,
                            *src_queue_type,
                        );
                    vk_buffer_barrier.dst_queue_family_index = self.inner.queue_family_index;
                }
                BarrierQueueTransition::None => {
                    vk_buffer_barrier.src_queue_family_index = ash::vk::QUEUE_FAMILY_IGNORED;
                    vk_buffer_barrier.dst_queue_family_index = ash::vk::QUEUE_FAMILY_IGNORED;
                }
            }

            src_access_flags |= vk_buffer_barrier.src_access_mask;
            dst_access_flags |= vk_buffer_barrier.dst_access_mask;

            vk_buffer_barriers.push(vk_buffer_barrier);
        }

        fn image_subresource_range(
            texture: &Texture,
            array_slice: Option<u16>,
            mip_slice: Option<u8>,
        ) -> ash::vk::ImageSubresourceRange {
            let mut subresource_range = ash::vk::ImageSubresourceRange::builder()
                .aspect_mask(texture.vk_aspect_mask())
                .build();

            if let Some(array_slice) = array_slice {
                subresource_range.layer_count = 1;
                subresource_range.base_array_layer = u32::from(array_slice);
                assert!(u32::from(array_slice) < texture.definition().array_length);
            } else {
                subresource_range.layer_count = ash::vk::REMAINING_ARRAY_LAYERS;
                subresource_range.base_array_layer = 0;
            };

            if let Some(mip_slice) = mip_slice {
                subresource_range.level_count = 1;
                subresource_range.base_mip_level = u32::from(mip_slice);
                assert!(u32::from(mip_slice) < texture.definition().mip_count);
            } else {
                subresource_range.level_count = ash::vk::REMAINING_MIP_LEVELS;
                subresource_range.base_mip_level = 0;
            }

            subresource_range
        }

        fn set_queue_family_indices(
            vk_image_barrier: &mut ash::vk::ImageMemoryBarrier,
            device_context: &DeviceContext,
            self_queue_family_index: u32,
            queue_transition: &BarrierQueueTransition,
        ) {
            match queue_transition {
                BarrierQueueTransition::ReleaseTo(dst_queue_type) => {
                    vk_image_barrier.src_queue_family_index = self_queue_family_index;
                    vk_image_barrier.dst_queue_family_index =
                        super::internal::queue_type_to_family_index(
                            device_context,
                            *dst_queue_type,
                        );
                }
                BarrierQueueTransition::AcquireFrom(src_queue_type) => {
                    vk_image_barrier.src_queue_family_index =
                        super::internal::queue_type_to_family_index(
                            device_context,
                            *src_queue_type,
                        );
                    vk_image_barrier.dst_queue_family_index = self_queue_family_index;
                }
                BarrierQueueTransition::None => {
                    vk_image_barrier.src_queue_family_index = ash::vk::QUEUE_FAMILY_IGNORED;
                    vk_image_barrier.dst_queue_family_index = ash::vk::QUEUE_FAMILY_IGNORED;
                }
            }
        }

        for barrier in texture_barriers {
            let subresource_range =
                image_subresource_range(barrier.texture, barrier.array_slice, barrier.mip_slice);

            // First transition is always from undefined. Doing it here can save downstream
            // code from having to implement a "first time" path and a "normal"
            // path
            let old_layout = if barrier.texture.take_is_undefined_layout() {
                ash::vk::ImageLayout::UNDEFINED
            } else {
                internal::resource_state_to_image_layout(barrier.src_state).unwrap()
            };

            let new_layout = internal::resource_state_to_image_layout(barrier.dst_state).unwrap();
            trace!(
                "Transition texture {:?} from {:?} to {:?}",
                barrier.texture,
                old_layout,
                new_layout
            );

            let mut vk_image_barrier = ash::vk::ImageMemoryBarrier::builder()
                .src_access_mask(super::internal::resource_state_to_access_flags(
                    barrier.src_state,
                ))
                .dst_access_mask(super::internal::resource_state_to_access_flags(
                    barrier.dst_state,
                ))
                .old_layout(old_layout)
                .new_layout(new_layout)
                .image(barrier.texture.vk_image())
                .subresource_range(subresource_range)
                .build();

            set_queue_family_indices(
                &mut vk_image_barrier,
                &self.inner.device_context,
                self.inner.queue_family_index,
                &barrier.queue_transition,
            );

            src_access_flags |= vk_image_barrier.src_access_mask;
            dst_access_flags |= vk_image_barrier.dst_access_mask;

            vk_image_barriers.push(vk_image_barrier);
        }

        let src_stage_mask = super::internal::determine_pipeline_stage_flags(
            self.inner.queue_type,
            src_access_flags,
        );
        let dst_stage_mask = super::internal::determine_pipeline_stage_flags(
            self.inner.queue_type,
            dst_access_flags,
        );

        if !vk_buffer_barriers.is_empty() || !vk_image_barriers.is_empty() {
            unsafe {
                self.inner.device_context.vk_device().cmd_pipeline_barrier(
                    self.inner.backend_command_buffer.vk_command_buffer,
                    src_stage_mask,
                    dst_stage_mask,
                    ash::vk::DependencyFlags::empty(),
                    &[],
                    &vk_buffer_barriers,
                    &vk_image_barriers,
                );
            }
        }
    }

    pub(crate) fn backend_cmd_fill_buffer(
        &mut self,
        dst_buffer: &Buffer,
        offset: u64,
        size: u64,
        data: u32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_fill_buffer(
                self.inner.backend_command_buffer.vk_command_buffer,
                dst_buffer.vk_buffer(),
                offset,
                size,
                data,
            );
        }
    }

    pub(crate) fn backend_cmd_copy_buffer_to_buffer(
        &mut self,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        copy_data: &[BufferCopy],
    ) {
        // todo(optimize)
        let vk_buffer_copy_regions = copy_data
            .iter()
            .map(|copy_data| {
                ash::vk::BufferCopy::builder()
                    .src_offset(copy_data.src_offset)
                    .dst_offset(copy_data.dst_offset)
                    .size(copy_data.size)
                    .build()
            })
            .collect::<Vec<_>>();
        unsafe {
            self.inner.device_context.vk_device().cmd_copy_buffer(
                self.inner.backend_command_buffer.vk_command_buffer,
                src_buffer.vk_buffer(),
                dst_buffer.vk_buffer(),
                &vk_buffer_copy_regions,
            );
        }
    }

    pub(crate) fn backend_cmd_copy_buffer_to_texture(
        &mut self,
        src_buffer: &Buffer,
        dst_texture: &Texture,
        params: &CmdCopyBufferToTextureParams,
    ) {
        let texture_def = dst_texture.definition();

        let width = 1.max(texture_def.extents.width >> params.mip_level);
        let height = 1.max(texture_def.extents.height >> params.mip_level);
        let depth = 1.max(texture_def.extents.depth >> params.mip_level);

        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_copy_buffer_to_image(
                    self.inner.backend_command_buffer.vk_command_buffer,
                    src_buffer.vk_buffer(),
                    dst_texture.vk_image(),
                    ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[ash::vk::BufferImageCopy {
                        image_extent: ash::vk::Extent3D {
                            width,
                            height,
                            depth,
                        },
                        image_offset: ash::vk::Offset3D { x: 0, y: 0, z: 0 },
                        image_subresource: ash::vk::ImageSubresourceLayers {
                            aspect_mask: dst_texture.vk_aspect_mask(),
                            mip_level: u32::from(params.mip_level),
                            base_array_layer: u32::from(params.array_layer),
                            layer_count: 1,
                        },
                        buffer_offset: params.buffer_offset,
                        buffer_image_height: 0,
                        buffer_row_length: 0,
                    }],
                );
        }
    }

    pub(crate) fn backend_cmd_blit_texture(
        &mut self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdBlitParams,
    ) {
        assert!(src_texture
            .definition()
            .usage_flags
            .intersects(ResourceUsage::AS_TRANSFERABLE));
        assert!(dst_texture
            .definition()
            .usage_flags
            .intersects(ResourceUsage::AS_TRANSFERABLE));

        let src_aspect_mask =
            super::internal::image_format_to_aspect_mask(src_texture.definition().format);
        let dst_aspect_mask =
            super::internal::image_format_to_aspect_mask(dst_texture.definition().format);

        let mut src_subresource = ash::vk::ImageSubresourceLayers::builder()
            .aspect_mask(src_aspect_mask)
            .mip_level(u32::from(params.src_mip_level))
            .build();
        let mut dst_subresource = ash::vk::ImageSubresourceLayers::builder()
            .aspect_mask(dst_aspect_mask)
            .mip_level(u32::from(params.dst_mip_level))
            .build();

        if let Some(array_slices) = params.array_slices {
            src_subresource.base_array_layer = u32::from(array_slices[0]);
            dst_subresource.base_array_layer = u32::from(array_slices[1]);
            src_subresource.layer_count = 1;
            dst_subresource.layer_count = 1;
        } else {
            src_subresource.base_array_layer = 0;
            dst_subresource.base_array_layer = 0;
            src_subresource.layer_count = ash::vk::REMAINING_ARRAY_LAYERS;
            dst_subresource.layer_count = ash::vk::REMAINING_ARRAY_LAYERS;
        }

        let src_offsets = [
            ash::vk::Offset3D {
                x: params.src_offsets[0].x as i32,
                y: params.src_offsets[0].y as i32,
                z: params.src_offsets[0].z as i32,
            },
            ash::vk::Offset3D {
                x: params.src_offsets[1].x as i32,
                y: params.src_offsets[1].y as i32,
                z: params.src_offsets[1].z as i32,
            },
        ];

        let dst_offsets = [
            ash::vk::Offset3D {
                x: params.dst_offsets[0].x as i32,
                y: params.dst_offsets[0].y as i32,
                z: params.dst_offsets[0].z as i32,
            },
            ash::vk::Offset3D {
                x: params.dst_offsets[1].x as i32,
                y: params.dst_offsets[1].y as i32,
                z: params.dst_offsets[1].z as i32,
            },
        ];

        let image_blit = ash::vk::ImageBlit::builder()
            .src_offsets(src_offsets)
            .src_subresource(src_subresource)
            .dst_offsets(dst_offsets)
            .dst_subresource(dst_subresource);

        unsafe {
            self.inner.device_context.vk_device().cmd_blit_image(
                self.inner.backend_command_buffer.vk_command_buffer,
                src_texture.vk_image(),
                super::internal::resource_state_to_image_layout(params.src_state).unwrap(),
                dst_texture.vk_image(),
                super::internal::resource_state_to_image_layout(params.dst_state).unwrap(),
                &[*image_blit],
                params.filtering.into(),
            );
        }
    }

    pub(crate) fn backend_cmd_copy_image(
        &mut self,
        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdCopyTextureParams,
    ) {
        assert!(src_texture
            .definition()
            .usage_flags
            .intersects(ResourceUsage::AS_TRANSFERABLE));
        assert!(dst_texture
            .definition()
            .usage_flags
            .intersects(ResourceUsage::AS_TRANSFERABLE));

        let src_aspect_mask = match params.src_plane_slice {
            PlaneSlice::Default => {
                super::internal::image_format_to_aspect_mask(src_texture.definition().format)
            }
            PlaneSlice::Depth => ash::vk::ImageAspectFlags::DEPTH,
            PlaneSlice::Stencil => ash::vk::ImageAspectFlags::STENCIL,
            PlaneSlice::Plane0 => ash::vk::ImageAspectFlags::PLANE_0,
            PlaneSlice::Plane1 => ash::vk::ImageAspectFlags::PLANE_1,
            PlaneSlice::Plane2 => ash::vk::ImageAspectFlags::PLANE_2,
        };

        let dst_aspect_mask = match params.dst_plane_slice {
            PlaneSlice::Default => {
                super::internal::image_format_to_aspect_mask(src_texture.definition().format)
            }
            PlaneSlice::Depth => ash::vk::ImageAspectFlags::DEPTH,
            PlaneSlice::Stencil => ash::vk::ImageAspectFlags::STENCIL,
            PlaneSlice::Plane0 => ash::vk::ImageAspectFlags::PLANE_0,
            PlaneSlice::Plane1 => ash::vk::ImageAspectFlags::PLANE_1,
            PlaneSlice::Plane2 => ash::vk::ImageAspectFlags::PLANE_2,
        };

        let mut src_subresource = ash::vk::ImageSubresourceLayers::builder()
            .aspect_mask(src_aspect_mask)
            .mip_level(u32::from(params.src_mip_level))
            .build();
        let mut dst_subresource = ash::vk::ImageSubresourceLayers::builder()
            .aspect_mask(dst_aspect_mask)
            .mip_level(u32::from(params.dst_mip_level))
            .build();

        src_subresource.base_array_layer = u32::from(params.src_array_slice);
        dst_subresource.base_array_layer = u32::from(params.dst_array_slice);
        src_subresource.layer_count = 1;
        dst_subresource.layer_count = 1;

        let src_offset = ash::vk::Offset3D {
            x: params.src_offset.x,
            y: params.src_offset.y,
            z: params.src_offset.z,
        };

        let dst_offset = ash::vk::Offset3D {
            x: params.dst_offset.x,
            y: params.dst_offset.y,
            z: params.dst_offset.z,
        };

        let image_copy = ash::vk::ImageCopy::builder()
            .src_offset(src_offset)
            .src_subresource(src_subresource)
            .dst_offset(dst_offset)
            .dst_subresource(dst_subresource)
            .extent(ash::vk::Extent3D {
                width: params.extent.width,
                height: params.extent.height,
                depth: params.extent.depth,
            });

        unsafe {
            self.inner.device_context.vk_device().cmd_copy_image(
                self.inner.backend_command_buffer.vk_command_buffer,
                src_texture.vk_image(),
                super::internal::resource_state_to_image_layout(params.src_state).unwrap(),
                dst_texture.vk_image(),
                super::internal::resource_state_to_image_layout(params.dst_state).unwrap(),
                &[*image_copy],
            );
        }
    }

    pub(crate) fn backend_begin_label(&mut self, label: &str) {
        let vk_command_buffer = self.vk_command_buffer();
        if let Some(debug_reporter) = &self.inner.backend_command_buffer.debug_reporter {
            debug_reporter.begin_label(vk_command_buffer, label);
        }
    }

    pub(crate) fn backend_end_label(&mut self) {
        let vk_command_buffer = self.vk_command_buffer();
        if let Some(debug_reporter) = &self.inner.backend_command_buffer.debug_reporter {
            debug_reporter.end_label(vk_command_buffer);
        }
    }
}
