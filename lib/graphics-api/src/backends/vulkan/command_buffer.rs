#![allow(clippy::too_many_lines)]
use std::{mem, ptr};

use ash::vk;

use super::{internal, VulkanDeviceContext, VulkanRootSignature};
use crate::{
    BarrierQueueTransition, BufferBarrier, BufferDrc, CmdBlitParams, CmdCopyBufferToTextureParams,
    CmdCopyTextureParams, ColorRenderTargetBinding, CommandBufferDef, CommandPool,
    DepthStencilRenderTargetBinding, DescriptorSetHandle, DeviceContextDrc, GfxResult,
    IndexBufferBinding, PipelineDrc, QueueType, ResourceState, ResourceUsage, RootSignatureDrc,
    TextureBarrier, TextureDrc, VertexBufferBinding,
};

pub(crate) struct VulkanCommandBuffer {
    vk_command_pool: vk::CommandPool,
    vk_command_buffer: vk::CommandBuffer,
}

impl VulkanCommandBuffer {
    pub fn new(
        command_pool: &CommandPool,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<Self> {
        let vk_command_pool = command_pool.platform_command_pool().vk_command_pool();
        log::trace!("Creating command buffers from pool {:?}", vk_command_pool);
        let command_buffer_level = if command_buffer_def.is_secondary {
            vk::CommandBufferLevel::SECONDARY
        } else {
            vk::CommandBufferLevel::PRIMARY
        };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(vk_command_pool)
            .level(command_buffer_level)
            .command_buffer_count(1);

        let vk_command_buffer = unsafe {
            command_pool
                .device_context()
                .platform_device()
                .allocate_command_buffers(&command_buffer_allocate_info)
        }?[0];

        Ok(Self {
            vk_command_pool,
            vk_command_buffer,
        })
    }

    pub fn vk_command_buffer(&self) -> vk::CommandBuffer {
        self.vk_command_buffer
    }

    pub fn begin(&self, device_context: &DeviceContextDrc) -> GfxResult<()> {
        //TODO: Use one-time-submit?
        let command_buffer_usage_flags = vk::CommandBufferUsageFlags::empty();

        let begin_info = vk::CommandBufferBeginInfo::builder().flags(command_buffer_usage_flags);

        unsafe {
            device_context
                .platform_device_context()
                .device()
                .begin_command_buffer(self.vk_command_buffer, &*begin_info)?;
        }

        Ok(())
    }

    pub fn end_command_buffer(&self, device_context: &DeviceContextDrc) -> GfxResult<()> {
        unsafe {
            device_context
                .platform_device_context()
                .device()
                .end_command_buffer(self.vk_command_buffer)?;
        }
        Ok(())
    }

    pub fn return_to_pool(&self, device_context: &DeviceContextDrc) {
        unsafe {
            device_context
                .platform_device_context()
                .device()
                .free_command_buffers(self.vk_command_pool, &[self.vk_command_buffer]);
        }
    }

    pub fn cmd_begin_render_pass(
        &self,
        device_context: &DeviceContextDrc,
        queue_type: QueueType,
        queue_family_index: u32,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: &Option<DepthStencilRenderTargetBinding<'_>>,
    ) -> GfxResult<()> {
        let (renderpass, framebuffer) = {
            let resource_cache = device_context.platform_device_context().resource_cache();
            let mut resource_cache = resource_cache.inner.lock().unwrap();

            let renderpass = resource_cache.renderpass_cache.get_or_create_renderpass(
                device_context,
                color_targets,
                depth_target.as_ref(),
            )?;
            let framebuffer = resource_cache.framebuffer_cache.get_or_create_framebuffer(
                device_context,
                &renderpass,
                color_targets,
                depth_target.as_ref(),
            )?;

            (renderpass, framebuffer)
        };

        let barriers = {
            let mut barriers = Vec::with_capacity(color_targets.len() + 1);
            for color_target in color_targets {
                if color_target
                    .texture_view
                    .texture()
                    .take_is_undefined_layout()
                {
                    log::trace!(
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
                    log::trace!(
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

        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: framebuffer.width(),
                height: framebuffer.height(),
            },
        };

        let mut clear_values = Vec::with_capacity(color_targets.len() + 1);
        for color_target in color_targets {
            clear_values.push(color_target.clear_value.into());
        }

        if let Some(depth_target) = &depth_target {
            clear_values.push(depth_target.clear_value.into());
        }

        if !barriers.is_empty() {
            self.cmd_resource_barrier(
                device_context,
                queue_type,
                queue_family_index,
                &[],
                &barriers,
            );
        }

        let begin_renderpass_create_info = vk::RenderPassBeginInfo::builder()
            .render_pass(renderpass.vk_renderpass())
            .framebuffer(framebuffer.vk_framebuffer())
            .render_area(render_area)
            .clear_values(&clear_values);

        unsafe {
            device_context.platform_device().cmd_begin_render_pass(
                self.vk_command_buffer,
                &*begin_renderpass_create_info,
                vk::SubpassContents::INLINE,
            );
        }

        #[allow(clippy::cast_precision_loss)]
        self.cmd_set_viewport(
            device_context,
            0.0,
            0.0,
            framebuffer.width() as f32,
            framebuffer.height() as f32,
            0.0,
            1.0,
        );
        self.cmd_set_scissor(
            device_context,
            0,
            0,
            framebuffer.width(),
            framebuffer.height(),
        );

        Ok(())
    }

    pub fn cmd_end_render_pass(&self, device_context: &DeviceContextDrc) {
        unsafe {
            device_context
                .platform_device()
                .cmd_end_render_pass(self.vk_command_buffer);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn cmd_set_viewport(
        &self,
        device_context: &DeviceContextDrc,
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
            device_context.platform_device().cmd_set_viewport(
                self.vk_command_buffer,
                0,
                &[vk::Viewport {
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

    pub fn cmd_set_scissor(
        &self,
        device_context: &DeviceContextDrc,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) {
        unsafe {
            device_context.platform_device().cmd_set_scissor(
                self.vk_command_buffer,
                0,
                &[vk::Rect2D {
                    offset: vk::Offset2D {
                        x: x.try_into().unwrap(),
                        y: y.try_into().unwrap(),
                    },
                    extent: vk::Extent2D { width, height },
                }],
            );
        }
    }

    pub fn cmd_set_stencil_reference_value(&self, device_context: &DeviceContextDrc, value: u32) {
        unsafe {
            device_context.platform_device().cmd_set_stencil_reference(
                self.vk_command_buffer,
                vk::StencilFaceFlags::FRONT_AND_BACK,
                value,
            );
        }
    }

    pub fn cmd_bind_pipeline(&self, device_context: &DeviceContextDrc, pipeline: &PipelineDrc) {
        //TODO: Add verification that the pipeline is compatible with the renderpass created by the targets
        let pipeline_bind_point =
            super::internal::pipeline_type_pipeline_bind_point(pipeline.pipeline_type());

        unsafe {
            device_context.platform_device().cmd_bind_pipeline(
                self.vk_command_buffer,
                pipeline_bind_point,
                pipeline.platform_pipeline().vk_pipeline(),
            );
        }
    }

    pub fn cmd_bind_vertex_buffers(
        &self,
        device_context: &DeviceContextDrc,
        first_binding: u32,
        bindings: &[VertexBufferBinding<'_>],
    ) {
        let mut buffers = Vec::with_capacity(bindings.len());
        let mut offsets = Vec::with_capacity(bindings.len());
        for binding in bindings {
            buffers.push(binding.buffer.platform_buffer().vk_buffer());
            offsets.push(binding.byte_offset);
        }

        unsafe {
            device_context.platform_device().cmd_bind_vertex_buffers(
                self.vk_command_buffer,
                first_binding,
                &buffers,
                &offsets,
            );
        }
    }

    pub fn cmd_bind_index_buffer(
        &self,
        device_context: &DeviceContextDrc,
        binding: &IndexBufferBinding<'_>,
    ) {
        unsafe {
            device_context.platform_device().cmd_bind_index_buffer(
                self.vk_command_buffer,
                binding.buffer.platform_buffer().vk_buffer(),
                binding.byte_offset,
                binding.index_type.into(),
            );
        }
    }

    pub fn cmd_bind_descriptor_set_handle(
        &self,
        device_context: &DeviceContextDrc,
        root_signature: &RootSignatureDrc,
        set_index: u32,
        descriptor_set_handle: DescriptorSetHandle,
    ) {
        let bind_point = root_signature.pipeline_type();

        unsafe {
            device_context.platform_device().cmd_bind_descriptor_sets(
                self.vk_command_buffer,
                super::internal::pipeline_type_pipeline_bind_point(bind_point),
                root_signature
                    .platform_root_signature()
                    .vk_pipeline_layout(),
                set_index,
                &[descriptor_set_handle.vk_type],
                &[],
            );
        }
    }

    pub fn cmd_push_constants<T: Sized>(
        &self,
        device_context: &DeviceContextDrc,
        root_signature: &VulkanRootSignature,
        constants: &T,
    ) {
        let constants_size = mem::size_of::<T>();
        let constants_ptr = (constants as *const T).cast::<u8>();
        unsafe {
            let data_slice = &*ptr::slice_from_raw_parts(constants_ptr, constants_size);
            device_context.platform_device().cmd_push_constants(
                self.vk_command_buffer,
                root_signature.vk_pipeline_layout(),
                vk::ShaderStageFlags::ALL,
                0,
                data_slice,
            );
        }
    }

    pub fn cmd_draw(
        &self,
        device_context: &DeviceContextDrc,
        vertex_count: u32,
        first_vertex: u32,
    ) {
        unsafe {
            device_context.platform_device().cmd_draw(
                self.vk_command_buffer,
                vertex_count,
                1,
                first_vertex,
                0,
            );
        }
    }

    pub fn cmd_draw_instanced(
        &self,
        device_context: &DeviceContextDrc,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        unsafe {
            device_context.platform_device().cmd_draw(
                self.vk_command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    pub fn cmd_draw_indexed(
        &self,
        device_context: &DeviceContextDrc,
        index_count: u32,
        first_index: u32,
        vertex_offset: i32,
    ) {
        unsafe {
            device_context.platform_device().cmd_draw_indexed(
                self.vk_command_buffer,
                index_count,
                1,
                first_index,
                vertex_offset,
                0,
            );
        }
    }

    pub fn cmd_draw_indexed_instanced(
        &self,
        device_context: &DeviceContextDrc,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) {
        unsafe {
            device_context.platform_device().cmd_draw_indexed(
                self.vk_command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    pub fn cmd_dispatch(
        &self,
        device_context: &DeviceContextDrc,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unsafe {
            device_context.platform_device().cmd_dispatch(
                self.vk_command_buffer,
                group_count_x,
                group_count_y,
                group_count_z,
            );
        }
    }

    pub fn cmd_resource_barrier(
        &self,
        device_context: &DeviceContextDrc,
        queue_type: QueueType,
        queue_family_index: u32,
        buffer_barriers: &[BufferBarrier<'_>],
        texture_barriers: &[TextureBarrier<'_>],
    ) {
        let mut vk_image_barriers = Vec::with_capacity(texture_barriers.len());
        let mut vk_buffer_barriers = Vec::with_capacity(buffer_barriers.len());

        let mut src_access_flags = vk::AccessFlags::empty();
        let mut dst_access_flags = vk::AccessFlags::empty();

        for barrier in buffer_barriers {
            let mut vk_buffer_barrier = vk::BufferMemoryBarrier::builder()
                .src_access_mask(super::internal::resource_state_to_access_flags(
                    barrier.src_state,
                ))
                .dst_access_mask(super::internal::resource_state_to_access_flags(
                    barrier.dst_state,
                ))
                .buffer(barrier.buffer.platform_buffer().vk_buffer())
                .size(vk::WHOLE_SIZE)
                .offset(0)
                .build();

            match &barrier.queue_transition {
                BarrierQueueTransition::ReleaseTo(dst_queue_type) => {
                    vk_buffer_barrier.src_queue_family_index = queue_family_index;
                    vk_buffer_barrier.dst_queue_family_index =
                        super::internal::queue_type_to_family_index(
                            device_context.platform_device_context(),
                            *dst_queue_type,
                        );
                }
                BarrierQueueTransition::AcquireFrom(src_queue_type) => {
                    vk_buffer_barrier.src_queue_family_index =
                        super::internal::queue_type_to_family_index(
                            device_context.platform_device_context(),
                            *src_queue_type,
                        );
                    vk_buffer_barrier.dst_queue_family_index = queue_family_index;
                }
                BarrierQueueTransition::None => {
                    vk_buffer_barrier.src_queue_family_index = vk::QUEUE_FAMILY_IGNORED;
                    vk_buffer_barrier.dst_queue_family_index = vk::QUEUE_FAMILY_IGNORED;
                }
            }

            src_access_flags |= vk_buffer_barrier.src_access_mask;
            dst_access_flags |= vk_buffer_barrier.dst_access_mask;

            vk_buffer_barriers.push(vk_buffer_barrier);
        }

        fn image_subresource_range(
            texture: &TextureDrc,
            array_slice: Option<u16>,
            mip_slice: Option<u8>,
        ) -> vk::ImageSubresourceRange {
            let mut subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(texture.platform_texture().vk_aspect_mask())
                .build();

            if let Some(array_slice) = array_slice {
                subresource_range.layer_count = 1;
                subresource_range.base_array_layer = u32::from(array_slice);
                assert!(u32::from(array_slice) < texture.definition().array_length);
            } else {
                subresource_range.layer_count = vk::REMAINING_ARRAY_LAYERS;
                subresource_range.base_array_layer = 0;
            };

            if let Some(mip_slice) = mip_slice {
                subresource_range.level_count = 1;
                subresource_range.base_mip_level = u32::from(mip_slice);
                assert!(u32::from(mip_slice) < texture.definition().mip_count);
            } else {
                subresource_range.level_count = vk::REMAINING_MIP_LEVELS;
                subresource_range.base_mip_level = 0;
            }

            subresource_range
        }

        fn set_queue_family_indices(
            vk_image_barrier: &mut vk::ImageMemoryBarrier,
            device_context: &VulkanDeviceContext,
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
                    vk_image_barrier.src_queue_family_index = vk::QUEUE_FAMILY_IGNORED;
                    vk_image_barrier.dst_queue_family_index = vk::QUEUE_FAMILY_IGNORED;
                }
            }
        }

        for barrier in texture_barriers {
            let subresource_range =
                image_subresource_range(barrier.texture, barrier.array_slice, barrier.mip_slice);

            // First transition is always from undefined. Doing it here can save downstream code
            // from having to implement a "first time" path and a "normal" path
            let old_layout = if barrier.texture.take_is_undefined_layout() {
                vk::ImageLayout::UNDEFINED
            } else {
                internal::resource_state_to_image_layout(barrier.src_state).unwrap()
            };

            let new_layout = internal::resource_state_to_image_layout(barrier.dst_state).unwrap();
            log::trace!(
                "Transition texture {:?} from {:?} to {:?}",
                barrier.texture,
                old_layout,
                new_layout
            );

            let mut vk_image_barrier = vk::ImageMemoryBarrier::builder()
                .src_access_mask(super::internal::resource_state_to_access_flags(
                    barrier.src_state,
                ))
                .dst_access_mask(super::internal::resource_state_to_access_flags(
                    barrier.dst_state,
                ))
                .old_layout(old_layout)
                .new_layout(new_layout)
                .image(barrier.texture.platform_texture().vk_image())
                .subresource_range(subresource_range)
                .build();

            set_queue_family_indices(
                &mut vk_image_barrier,
                device_context.platform_device_context(),
                queue_family_index,
                &barrier.queue_transition,
            );

            src_access_flags |= vk_image_barrier.src_access_mask;
            dst_access_flags |= vk_image_barrier.dst_access_mask;

            vk_image_barriers.push(vk_image_barrier);
        }

        let src_stage_mask =
            super::internal::determine_pipeline_stage_flags(queue_type, src_access_flags);
        let dst_stage_mask =
            super::internal::determine_pipeline_stage_flags(queue_type, dst_access_flags);

        if !vk_buffer_barriers.is_empty() || !vk_image_barriers.is_empty() {
            unsafe {
                device_context.platform_device().cmd_pipeline_barrier(
                    self.vk_command_buffer,
                    src_stage_mask,
                    dst_stage_mask,
                    vk::DependencyFlags::empty(),
                    &[],
                    &vk_buffer_barriers,
                    &vk_image_barriers,
                );
            }
        }
    }

    pub fn cmd_copy_buffer_to_buffer(
        &self,
        device_context: &DeviceContextDrc,
        src_buffer: &BufferDrc,
        dst_buffer: &BufferDrc,
        src_offset: u64,
        dst_offset: u64,
        size: u64,
    ) {
        unsafe {
            device_context.platform_device().cmd_copy_buffer(
                self.vk_command_buffer,
                src_buffer.platform_buffer().vk_buffer(),
                dst_buffer.platform_buffer().vk_buffer(),
                &[vk::BufferCopy {
                    src_offset,
                    dst_offset,
                    size,
                }],
            );
        }
    }

    pub fn cmd_copy_buffer_to_texture(
        &self,
        device_context: &DeviceContextDrc,
        src_buffer: &BufferDrc,
        dst_texture: &TextureDrc,
        params: &CmdCopyBufferToTextureParams,
    ) {
        let texture_def = dst_texture.definition();

        let width = 1.max(texture_def.extents.width >> params.mip_level);
        let height = 1.max(texture_def.extents.height >> params.mip_level);
        let depth = 1.max(texture_def.extents.depth >> params.mip_level);

        unsafe {
            device_context.platform_device().cmd_copy_buffer_to_image(
                self.vk_command_buffer,
                src_buffer.platform_buffer().vk_buffer(),
                dst_texture.platform_texture().vk_image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[vk::BufferImageCopy {
                    image_extent: vk::Extent3D {
                        width,
                        height,
                        depth,
                    },
                    image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                    image_subresource: vk::ImageSubresourceLayers {
                        aspect_mask: dst_texture.platform_texture().vk_aspect_mask(),
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

    pub fn cmd_blit_texture(
        &self,
        device_context: &DeviceContextDrc,
        src_texture: &TextureDrc,
        dst_texture: &TextureDrc,
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

        let mut src_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(src_aspect_mask)
            .mip_level(u32::from(params.src_mip_level))
            .build();
        let mut dst_subresource = vk::ImageSubresourceLayers::builder()
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
            src_subresource.layer_count = vk::REMAINING_ARRAY_LAYERS;
            dst_subresource.layer_count = vk::REMAINING_ARRAY_LAYERS;
        }

        let src_offsets = [
            vk::Offset3D {
                x: params.src_offsets[0].x as i32,
                y: params.src_offsets[0].y as i32,
                z: params.src_offsets[0].z as i32,
            },
            vk::Offset3D {
                x: params.src_offsets[1].x as i32,
                y: params.src_offsets[1].y as i32,
                z: params.src_offsets[1].z as i32,
            },
        ];

        let dst_offsets = [
            vk::Offset3D {
                x: params.dst_offsets[0].x as i32,
                y: params.dst_offsets[0].y as i32,
                z: params.dst_offsets[0].z as i32,
            },
            vk::Offset3D {
                x: params.dst_offsets[1].x as i32,
                y: params.dst_offsets[1].y as i32,
                z: params.dst_offsets[1].z as i32,
            },
        ];

        let image_blit = vk::ImageBlit::builder()
            .src_offsets(src_offsets)
            .src_subresource(src_subresource)
            .dst_offsets(dst_offsets)
            .dst_subresource(dst_subresource);

        unsafe {
            device_context.platform_device().cmd_blit_image(
                self.vk_command_buffer,
                src_texture.platform_texture().vk_image(),
                super::internal::resource_state_to_image_layout(params.src_state).unwrap(),
                dst_texture.platform_texture().vk_image(),
                super::internal::resource_state_to_image_layout(params.dst_state).unwrap(),
                &[*image_blit],
                params.filtering.into(),
            );
        }
    }

    pub fn cmd_copy_image(
        &self,
        device_context: &DeviceContextDrc,
        src_texture: &TextureDrc,
        dst_texture: &TextureDrc,
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

        let src_aspect_mask =
            super::internal::image_format_to_aspect_mask(src_texture.definition().format);
        let dst_aspect_mask =
            super::internal::image_format_to_aspect_mask(dst_texture.definition().format);

        let mut src_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(src_aspect_mask)
            .mip_level(u32::from(params.src_mip_level))
            .build();
        let mut dst_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(dst_aspect_mask)
            .mip_level(u32::from(params.dst_mip_level))
            .build();

        src_subresource.base_array_layer = u32::from(params.src_array_slice);
        dst_subresource.base_array_layer = u32::from(params.dst_array_slice);
        src_subresource.layer_count = 1;
        dst_subresource.layer_count = 1;

        let src_offset = vk::Offset3D {
            x: params.src_offset.x,
            y: params.src_offset.y,
            z: params.src_offset.z,
        };

        let dst_offset = vk::Offset3D {
            x: params.dst_offset.x,
            y: params.dst_offset.y,
            z: params.dst_offset.z,
        };

        let image_copy = vk::ImageCopy::builder()
            .src_offset(src_offset)
            .src_subresource(src_subresource)
            .dst_offset(dst_offset)
            .dst_subresource(dst_subresource)
            .extent(vk::Extent3D {
                width: params.extent.width,
                height: params.extent.height,
                depth: params.extent.depth,
            });

        unsafe {
            device_context.platform_device().cmd_copy_image(
                self.vk_command_buffer,
                src_texture.platform_texture().vk_image(),
                super::internal::resource_state_to_image_layout(params.src_state).unwrap(),
                dst_texture.platform_texture().vk_image(),
                super::internal::resource_state_to_image_layout(params.dst_state).unwrap(),
                &[*image_copy],
            );
        }
    }
}
