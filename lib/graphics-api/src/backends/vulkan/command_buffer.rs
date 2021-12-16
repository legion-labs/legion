#![allow(clippy::too_many_lines)]
use std::{mem, ptr};

use super::internal;
use crate::{
    BarrierQueueTransition, Buffer, BufferBarrier, CmdBlitParams, CmdCopyBufferToTextureParams,
    CmdCopyTextureParams, ColorRenderTargetBinding, CommandBuffer, CommandBufferDef, CommandPool,
    DepthStencilRenderTargetBinding, DescriptorSetHandle, DeviceContext, GfxResult,
    IndexBufferBinding, Pipeline, PipelineType, ResourceState, ResourceUsage, RootSignature,
    Texture, TextureBarrier, VertexBufferBinding,
};
pub(crate) struct VulkanCommandBuffer {
    vk_command_buffer: ash::vk::CommandBuffer,
}

impl VulkanCommandBuffer {
    pub(crate) fn new(
        command_pool: &CommandPool,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<Self> {
        let vk_command_pool = command_pool.vk_command_pool();
        log::trace!("Creating command buffers from pool {:?}", vk_command_pool);
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

        Ok(Self { vk_command_buffer })
    }
}

impl CommandBuffer {
    pub(crate) fn vk_command_buffer(&self) -> ash::vk::CommandBuffer {
        self.inner.platform_command_buffer.vk_command_buffer
    }

    pub(crate) fn begin_platform(&self) -> GfxResult<()> {
        // TODO: check if it is not a ONE TIME SUBMIT
        let command_buffer_usage_flags = ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT;

        let begin_info =
            ash::vk::CommandBufferBeginInfo::builder().flags(command_buffer_usage_flags);

        unsafe {
            self.inner.device_context.vk_device().begin_command_buffer(
                self.inner.platform_command_buffer.vk_command_buffer,
                &*begin_info,
            )?;
        }

        Ok(())
    }

    pub(crate) fn end_platform(&self) -> GfxResult<()> {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .end_command_buffer(self.inner.platform_command_buffer.vk_command_buffer)?;
        }
        Ok(())
    }

    pub(crate) fn cmd_begin_render_pass_platform(
        &self,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: &Option<DepthStencilRenderTargetBinding<'_>>,
    ) -> GfxResult<()> {
        let (renderpass, framebuffer) = {
            let resource_cache = self.inner.device_context.resource_cache();
            let mut resource_cache = resource_cache.inner.lock().unwrap();

            let renderpass = resource_cache.renderpass_cache.get_or_create_renderpass(
                &self.inner.device_context,
                color_targets,
                depth_target.as_ref(),
            )?;
            let framebuffer = resource_cache.framebuffer_cache.get_or_create_framebuffer(
                &self.inner.device_context,
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

        let render_area = ash::vk::Rect2D {
            offset: ash::vk::Offset2D { x: 0, y: 0 },
            extent: ash::vk::Extent2D {
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
            self.cmd_resource_barrier_platform(&[], &barriers);
        }

        let begin_renderpass_create_info = ash::vk::RenderPassBeginInfo::builder()
            .render_pass(renderpass.vk_renderpass())
            .framebuffer(framebuffer.vk_framebuffer())
            .render_area(render_area)
            .clear_values(&clear_values);

        unsafe {
            self.inner.device_context.vk_device().cmd_begin_render_pass(
                self.inner.platform_command_buffer.vk_command_buffer,
                &*begin_renderpass_create_info,
                ash::vk::SubpassContents::INLINE,
            );
        }

        #[allow(clippy::cast_precision_loss)]
        self.cmd_set_viewport(
            0.0,
            0.0,
            framebuffer.width() as f32,
            framebuffer.height() as f32,
            0.0,
            1.0,
        )?;
        self.cmd_set_scissor(0, 0, framebuffer.width(), framebuffer.height())?;

        Ok(())
    }

    pub(crate) fn cmd_end_render_pass_platform(&self) {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_end_render_pass(self.inner.platform_command_buffer.vk_command_buffer);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cmd_set_viewport_platform(
        &self,
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
                self.inner.platform_command_buffer.vk_command_buffer,
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

    pub(crate) fn cmd_set_scissor_platform(&self, x: u32, y: u32, width: u32, height: u32) {
        unsafe {
            self.inner.device_context.vk_device().cmd_set_scissor(
                self.inner.platform_command_buffer.vk_command_buffer,
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

    pub(crate) fn cmd_set_stencil_reference_value_platform(&self, value: u32) {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .cmd_set_stencil_reference(
                    self.inner.platform_command_buffer.vk_command_buffer,
                    ash::vk::StencilFaceFlags::FRONT_AND_BACK,
                    value,
                );
        }
    }

    pub(crate) fn cmd_bind_pipeline_platform(&self, pipeline: &Pipeline) {
        //TODO: Add verification that the pipeline is compatible with the renderpass created by the targets
        let pipeline_bind_point =
            super::internal::pipeline_type_pipeline_bind_point(pipeline.pipeline_type());

        unsafe {
            self.inner.device_context.vk_device().cmd_bind_pipeline(
                self.inner.platform_command_buffer.vk_command_buffer,
                pipeline_bind_point,
                pipeline.vk_pipeline(),
            );
        }
    }

    pub(crate) fn cmd_bind_vertex_buffers_platform(
        &self,
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
                    self.inner.platform_command_buffer.vk_command_buffer,
                    first_binding,
                    &buffers,
                    &offsets,
                );
        }
    }

    pub(crate) fn cmd_bind_index_buffer_platform(&self, binding: &IndexBufferBinding<'_>) {
        unsafe {
            self.inner.device_context.vk_device().cmd_bind_index_buffer(
                self.inner.platform_command_buffer.vk_command_buffer,
                binding.buffer.vk_buffer(),
                binding.byte_offset,
                binding.index_type.into(),
            );
        }
    }

    pub(crate) fn cmd_bind_descriptor_set_handle_platform(
        &self,
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
                    self.inner.platform_command_buffer.vk_command_buffer,
                    super::internal::pipeline_type_pipeline_bind_point(pipeline_type),
                    root_signature.vk_pipeline_layout(),
                    set_index,
                    &[descriptor_set_handle.vk_type],
                    &[],
                );
        }
    }

    pub(crate) fn cmd_push_constants_platform<T: Sized>(
        &self,
        root_signature: &RootSignature,
        constants: &T,
    ) {
        let constants_size = mem::size_of::<T>();
        let constants_ptr = (constants as *const T).cast::<u8>();
        unsafe {
            let data_slice = &*ptr::slice_from_raw_parts(constants_ptr, constants_size);
            self.inner.device_context.vk_device().cmd_push_constants(
                self.inner.platform_command_buffer.vk_command_buffer,
                root_signature.vk_pipeline_layout(),
                ash::vk::ShaderStageFlags::ALL,
                0,
                data_slice,
            );
        }
    }

    pub(crate) fn cmd_draw_platform(&self, vertex_count: u32, first_vertex: u32) {
        unsafe {
            self.inner.device_context.vk_device().cmd_draw(
                self.inner.platform_command_buffer.vk_command_buffer,
                vertex_count,
                1,
                first_vertex,
                0,
            );
        }
    }

    pub(crate) fn cmd_draw_instanced_platform(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_draw(
                self.inner.platform_command_buffer.vk_command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    pub(crate) fn cmd_draw_indexed_platform(
        &self,
        index_count: u32,
        first_index: u32,
        vertex_offset: i32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_draw_indexed(
                self.inner.platform_command_buffer.vk_command_buffer,
                index_count,
                1,
                first_index,
                vertex_offset,
                0,
            );
        }
    }

    pub(crate) fn cmd_draw_indexed_instanced_platform(
        &self,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_draw_indexed(
                self.inner.platform_command_buffer.vk_command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    pub(crate) fn cmd_dispatch_platform(
        &self,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_dispatch(
                self.inner.platform_command_buffer.vk_command_buffer,
                group_count_x,
                group_count_y,
                group_count_z,
            );
        }
    }

    pub(crate) fn cmd_resource_barrier_platform(
        &self,
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

            // First transition is always from undefined. Doing it here can save downstream code
            // from having to implement a "first time" path and a "normal" path
            let old_layout = if barrier.texture.take_is_undefined_layout() {
                ash::vk::ImageLayout::UNDEFINED
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
                    self.inner.platform_command_buffer.vk_command_buffer,
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

    pub(crate) fn cmd_copy_buffer_to_buffer_platform(
        &self,

        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        copy_data: &[ash::vk::BufferCopy],
    ) {
        unsafe {
            self.inner.device_context.vk_device().cmd_copy_buffer(
                self.inner.platform_command_buffer.vk_command_buffer,
                src_buffer.vk_buffer(),
                dst_buffer.vk_buffer(),
                copy_data,
            );
        }
    }

    pub(crate) fn cmd_copy_buffer_to_texture_platform(
        &self,

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
                    self.inner.platform_command_buffer.vk_command_buffer,
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

    pub(crate) fn cmd_blit_texture_platform(
        &self,

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
                self.inner.platform_command_buffer.vk_command_buffer,
                src_texture.vk_image(),
                super::internal::resource_state_to_image_layout(params.src_state).unwrap(),
                dst_texture.vk_image(),
                super::internal::resource_state_to_image_layout(params.dst_state).unwrap(),
                &[*image_blit],
                params.filtering.into(),
            );
        }
    }

    pub(crate) fn cmd_copy_image_platform(
        &self,

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
                self.inner.platform_command_buffer.vk_command_buffer,
                src_texture.vk_image(),
                super::internal::resource_state_to_image_layout(params.src_state).unwrap(),
                dst_texture.vk_image(),
                super::internal::resource_state_to_image_layout(params.dst_state).unwrap(),
                &[*image_copy],
            );
        }
    }
}
