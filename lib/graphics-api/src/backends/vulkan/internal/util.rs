use crate::backends::vulkan::VulkanDeviceContext;
use crate::{BlendFactor, BlendState, BlendStateRenderTarget, BlendStateTargets, DepthState, Format, PipelineType, QueueType, RasterizerState, ResourceState, ResourceType, ShaderResourceType, ResourceUsage};
use ash::vk;

pub(crate) fn pipeline_type_pipeline_bind_point(
    pipeline_type: PipelineType,
) -> vk::PipelineBindPoint {
    match pipeline_type {
        PipelineType::Graphics => vk::PipelineBindPoint::GRAPHICS,
        PipelineType::Compute => vk::PipelineBindPoint::COMPUTE,
    }
}

pub(crate) fn resource_type_buffer_usage_flags(
    resource_usage : ResourceUsage,
    // resource_type: ResourceType,
    // has_format: bool,
) -> vk::BufferUsageFlags {
    let mut usage_flags = vk::BufferUsageFlags::TRANSFER_SRC;

    if resource_usage.intersects(ResourceUsage::HAS_CONST_BUFFER_VIEW) {
        usage_flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
    }

    if resource_usage.intersects(ResourceUsage::HAS_UNORDERED_ACCESS_VIEW|ResourceUsage::HAS_SHADER_RESOURCE_VIEW ) {
        usage_flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        // if has_format {
        //     usage_flags |= vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER;
        // }
    }

    // if resource_type.intersects(ResourceUsage::BUFFER) {
    //     usage_flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
    //     // if has_format {
    //     //     usage_flags |= vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER;
    //     // }
    // }

    if resource_usage.intersects(ResourceUsage::HAS_INDEX_BUFFER) {
        usage_flags |= vk::BufferUsageFlags::INDEX_BUFFER;
    }

    if resource_usage.intersects(ResourceUsage::HAS_VERTEX_BUFFER) {
        usage_flags |= vk::BufferUsageFlags::VERTEX_BUFFER;
    }

    if resource_usage.intersects(ResourceUsage::HAS_INDIRECT_BUFFER) {
        usage_flags |= vk::BufferUsageFlags::INDIRECT_BUFFER;
    }

    usage_flags
}

pub(crate) fn resource_type_image_usage_flags(resource_type: ResourceType) -> vk::ImageUsageFlags {
    let mut usage_flags = vk::ImageUsageFlags::empty();

    if resource_type.intersects(ResourceType::TEXTURE) {
        usage_flags |= vk::ImageUsageFlags::SAMPLED;
    }

    if resource_type.intersects(ResourceType::TEXTURE_READ_WRITE) {
        usage_flags |= vk::ImageUsageFlags::STORAGE;
    }

    usage_flags
}

pub(crate) fn image_format_to_aspect_mask(
    format: Format, /*, include_stencil: bool*/
) -> vk::ImageAspectFlags {
    match format {
        // Depth only
        Format::D16_UNORM | Format::X8_D24_UNORM_PACK32 | Format::D32_SFLOAT => {
            vk::ImageAspectFlags::DEPTH
        }
        // Stencil only
        Format::S8_UINT => vk::ImageAspectFlags::STENCIL,
        // Both
        Format::D16_UNORM_S8_UINT | Format::D24_UNORM_S8_UINT | Format::D32_SFLOAT_S8_UINT => {
            //if include_stencil {
            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            //} else {
            //    vk::ImageAspectFlags::DEPTH
            //}
        }
        // Otherwise assume color
        _ => vk::ImageAspectFlags::COLOR,
    }
}

pub fn shader_resource_type_to_descriptor_type(shader_resource_type: ShaderResourceType) -> Option<vk::DescriptorType> {
    match shader_resource_type {
        ShaderResourceType::Sampler => Some(vk::DescriptorType::SAMPLER),
        ShaderResourceType::ConstantBuffer(_e) => Some(vk::DescriptorType::UNIFORM_BUFFER),
        ShaderResourceType::ShaderResourceView(_e) => todo!(),
        ShaderResourceType::UnorderedAccessView(_e) => todo!(),
        ShaderResourceType::Undefined => todo!(),
        
    
    // pub const SAMPLED_IMAGE: Self = Self(2);
    // pub const STORAGE_IMAGE: Self = Self(3);        
    // pub const STORAGE_BUFFER: Self = Self(7);    
    }    
}

pub(crate) fn resource_state_to_access_flags(state: ResourceState) -> vk::AccessFlags {
    let mut flags = vk::AccessFlags::empty();
    if state.intersects(ResourceState::COPY_SRC) {
        flags |= vk::AccessFlags::TRANSFER_READ;
    }

    if state.intersects(ResourceState::COPY_DST) {
        flags |= vk::AccessFlags::TRANSFER_WRITE;
    }

    if state.intersects(ResourceState::VERTEX_AND_CONSTANT_BUFFER) {
        flags |= vk::AccessFlags::UNIFORM_READ | vk::AccessFlags::VERTEX_ATTRIBUTE_READ;
    }

    if state.intersects(ResourceState::INDEX_BUFFER) {
        flags |= vk::AccessFlags::INDEX_READ;
    }

    if state.intersects(ResourceState::UNORDERED_ACCESS) {
        flags |= vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE;
    }

    if state.intersects(ResourceState::INDIRECT_ARGUMENT) {
        flags |= vk::AccessFlags::INDIRECT_COMMAND_READ;
    }

    if state.intersects(ResourceState::RENDER_TARGET) {
        flags |= vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
    }

    if state.intersects(ResourceState::DEPTH_WRITE) {
        flags |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
    }

    if state.intersects(ResourceState::SHADER_RESOURCE) {
        flags |= vk::AccessFlags::SHADER_READ;
    }

    if state.intersects(ResourceState::PRESENT) {
        flags |= vk::AccessFlags::MEMORY_READ;
    }

    flags
}

pub(crate) fn resource_state_to_image_layout(state: ResourceState) -> Option<vk::ImageLayout> {
    if state.intersects(ResourceState::COPY_SRC) {
        Some(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
    } else if state.intersects(ResourceState::COPY_DST) {
        Some(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
    } else if state.intersects(ResourceState::RENDER_TARGET) {
        Some(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
    } else if state.intersects(ResourceState::DEPTH_WRITE) {
        Some(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
    } else if state.intersects(ResourceState::UNORDERED_ACCESS) {
        Some(vk::ImageLayout::GENERAL)
    } else if state.intersects(ResourceState::SHADER_RESOURCE) {
        Some(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
    } else if state.intersects(ResourceState::PRESENT) {
        Some(vk::ImageLayout::PRESENT_SRC_KHR)
    } else if state.intersects(ResourceState::COMMON) {
        Some(vk::ImageLayout::GENERAL)
    } else if state == ResourceState::UNDEFINED {
        Some(vk::ImageLayout::UNDEFINED)
    } else {
        None
    }
}

pub(crate) fn queue_type_to_family_index(
    device_context: &VulkanDeviceContext,
    queue_type: QueueType,
) -> u32 {
    match queue_type {
        QueueType::Graphics => {
            device_context
                .queue_family_indices()
                .graphics_queue_family_index
        }
        QueueType::Compute => {
            device_context
                .queue_family_indices()
                .compute_queue_family_index
        }
        QueueType::Transfer => {
            device_context
                .queue_family_indices()
                .transfer_queue_family_index
        }
    }
}

// Based on what is being accessed, determine what stages need to be blocked
pub(crate) fn determine_pipeline_stage_flags(
    queue_type: QueueType,
    access_flags: vk::AccessFlags,
) -> vk::PipelineStageFlags {
    let mut flags = vk::PipelineStageFlags::empty();
    match queue_type {
        QueueType::Graphics => {
            if access_flags
                .intersects(vk::AccessFlags::INDEX_READ | vk::AccessFlags::VERTEX_ATTRIBUTE_READ)
            {
                flags |= vk::PipelineStageFlags::VERTEX_INPUT;
            }

            if access_flags.intersects(
                vk::AccessFlags::UNIFORM_READ
                    | vk::AccessFlags::SHADER_READ
                    | vk::AccessFlags::SHADER_WRITE,
            ) {
                flags |= vk::PipelineStageFlags::VERTEX_INPUT;
                flags |= vk::PipelineStageFlags::FRAGMENT_SHADER;
                flags |= vk::PipelineStageFlags::COMPUTE_SHADER;

                // Not supported
                //flags |= vk::PipelineStageFlags::GEOMETRY_SHADER;
                //flags |= vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER;
                //flags |= vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER;
                // raytracing
            }

            if access_flags.intersects(vk::AccessFlags::INPUT_ATTACHMENT_READ) {
                flags |= vk::PipelineStageFlags::FRAGMENT_SHADER;
            }

            if access_flags.intersects(
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ) {
                flags |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
            }

            if access_flags.intersects(
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ) {
                flags |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
            }
        }
        QueueType::Compute => {
            if access_flags.intersects(
                vk::AccessFlags::INDEX_READ
                    | vk::AccessFlags::VERTEX_ATTRIBUTE_READ
                    | vk::AccessFlags::INPUT_ATTACHMENT_READ
                    | vk::AccessFlags::COLOR_ATTACHMENT_READ
                    | vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ) {
                return vk::PipelineStageFlags::ALL_COMMANDS;
            }

            if access_flags.intersects(
                vk::AccessFlags::UNIFORM_READ
                    | vk::AccessFlags::SHADER_READ
                    | vk::AccessFlags::SHADER_WRITE,
            ) {
                flags |= vk::PipelineStageFlags::COMPUTE_SHADER;
            }
        }
        QueueType::Transfer => {
            return vk::PipelineStageFlags::ALL_COMMANDS;
        }
    }

    //
    // Logic for both graphics/compute
    //
    if access_flags.intersects(vk::AccessFlags::INDIRECT_COMMAND_READ) {
        flags |= vk::PipelineStageFlags::DRAW_INDIRECT;
    }

    if access_flags.intersects(vk::AccessFlags::TRANSFER_READ | vk::AccessFlags::TRANSFER_WRITE) {
        flags |= vk::PipelineStageFlags::TRANSFER;
    }

    if access_flags.intersects(vk::AccessFlags::HOST_READ | vk::AccessFlags::HOST_WRITE) {
        flags |= vk::PipelineStageFlags::HOST;
    }

    if flags.is_empty() {
        flags |= vk::PipelineStageFlags::TOP_OF_PIPE;
    }

    flags
}

pub(crate) fn depth_state_to_create_info(
    depth_state: &DepthState,
) -> vk::PipelineDepthStencilStateCreateInfo {
    let front = vk::StencilOpState::builder()
        .fail_op(depth_state.front_stencil_fail_op.into())
        .pass_op(depth_state.front_stencil_pass_op.into())
        .depth_fail_op(depth_state.front_depth_fail_op.into())
        .compare_op(depth_state.front_stencil_compare_op.into())
        .compare_mask(depth_state.stencil_read_mask as u32)
        .write_mask(depth_state.stencil_write_mask as u32)
        .reference(0);

    let back = vk::StencilOpState::builder()
        .fail_op(depth_state.back_stencil_fail_op.into())
        .pass_op(depth_state.back_stencil_pass_op.into())
        .depth_fail_op(depth_state.back_depth_fail_op.into())
        .compare_op(depth_state.back_stencil_compare_op.into())
        .compare_mask(depth_state.stencil_read_mask as u32)
        .write_mask(depth_state.stencil_write_mask as u32)
        .reference(0);

    vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(depth_state.depth_test_enable)
        .depth_write_enable(depth_state.depth_write_enable)
        .depth_compare_op(depth_state.depth_compare_op.into())
        .depth_bounds_test_enable(false)
        .stencil_test_enable(depth_state.stencil_test_enable)
        .min_depth_bounds(0.0)
        .max_depth_bounds(1.0)
        .front(*front)
        .back(*back)
        .build()
}

pub(crate) fn rasterizer_state_to_create_info(
    rasterizer_state: &RasterizerState,
) -> vk::PipelineRasterizationStateCreateInfo {
    vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(rasterizer_state.depth_clamp_enable)
        .rasterizer_discard_enable(false)
        .polygon_mode(rasterizer_state.fill_mode.into())
        .cull_mode(rasterizer_state.cull_mode.into())
        .front_face(rasterizer_state.front_face.into())
        .depth_bias_enable(rasterizer_state.depth_bias != 0)
        .depth_bias_constant_factor(rasterizer_state.depth_bias as f32)
        .depth_bias_clamp(0.0)
        .depth_bias_slope_factor(rasterizer_state.depth_bias_slope_scaled)
        .line_width(1.0)
        .build()
}

//WARNING: This struct has pointers into the attachments vector. Don't mutate or drop the
// attachments vector
pub struct BlendStateVkCreateInfo {
    _attachments: Vec<vk::PipelineColorBlendAttachmentState>,
    blend_state: vk::PipelineColorBlendStateCreateInfo,
}

impl BlendStateVkCreateInfo {
    pub fn blend_state(&self) -> &vk::PipelineColorBlendStateCreateInfo {
        &self.blend_state
    }
}

pub(crate) fn blend_state_render_target_to_create_info(
    blend_state_rt: &BlendStateRenderTarget,
) -> vk::PipelineColorBlendAttachmentState {
    let blend_enable = blend_state_rt.src_factor != BlendFactor::One
        || blend_state_rt.src_factor_alpha != BlendFactor::One
        || blend_state_rt.dst_factor != BlendFactor::Zero
        || blend_state_rt.dst_factor_alpha != BlendFactor::Zero;

    vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(blend_enable)
        .color_write_mask(blend_state_rt.masks.into())
        .src_color_blend_factor(blend_state_rt.src_factor.into())
        .src_alpha_blend_factor(blend_state_rt.src_factor_alpha.into())
        .dst_color_blend_factor(blend_state_rt.dst_factor.into())
        .dst_alpha_blend_factor(blend_state_rt.dst_factor_alpha.into())
        .color_blend_op(blend_state_rt.blend_op.into())
        .alpha_blend_op(blend_state_rt.blend_op_alpha.into())
        .build()
}

pub fn blend_state_to_create_info(
    blend_state: &BlendState,
    color_attachment_count: usize,
) -> BlendStateVkCreateInfo {
    let mut blend_attachments_states = vec![];

    blend_state.verify(color_attachment_count);

    if let Some(first_attachment) = blend_state.render_target_blend_states.first() {
        for attachment_index in 0..color_attachment_count {
            let attachment_state = if blend_state
                .render_target_mask
                .intersects(BlendStateTargets::from_bits(1 << attachment_index).unwrap())
            {
                if blend_state.independent_blend {
                    blend_state_render_target_to_create_info(
                        &blend_state.render_target_blend_states[attachment_index],
                    )
                } else {
                    blend_state_render_target_to_create_info(first_attachment)
                }
            } else {
                vk::PipelineColorBlendAttachmentState::default()
            };

            blend_attachments_states.push(attachment_state);
        }
    }

    let blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::CLEAR)
        .attachments(&blend_attachments_states)
        .blend_constants([0.0, 0.0, 0.0, 0.0])
        .build();

    BlendStateVkCreateInfo {
        _attachments: blend_attachments_states,
        blend_state: blend_state_create_info,
    }
}
