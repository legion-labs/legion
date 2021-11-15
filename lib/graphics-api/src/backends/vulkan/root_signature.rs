use std::cmp;

use ash::vk;

use super::VulkanDeviceContext;
use crate::{GfxResult, RootSignatureDef, MAX_DESCRIPTOR_SET_LAYOUTS};

// Not currently exposed
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct DynamicDescriptorIndex(pub(crate) u32);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct PushConstantIndex(pub(crate) u32);

#[derive(Debug)]
pub(crate) struct VulkanRootSignature {
    pipeline_layout: vk::PipelineLayout,
}

impl VulkanRootSignature {
    pub(crate) fn new(
        device_context: &VulkanDeviceContext,
        definition: &RootSignatureDef,
    ) -> GfxResult<Self> {
        log::trace!("Create VulkanRootSignature");

        //
        // Create pipeline layout
        //
        let mut vk_descriptor_set_layouts =
            [vk::DescriptorSetLayout::null(); MAX_DESCRIPTOR_SET_LAYOUTS];

        let mut descriptor_set_layout_count = 0;
        for layout in &definition.descriptor_set_layouts {
            let set_index = layout.set_index() as usize;
            vk_descriptor_set_layouts[set_index] = layout.platform_layout().vk_layout();
            descriptor_set_layout_count = cmp::max(descriptor_set_layout_count, set_index + 1);
        }

        let mut push_constant_ranges = Vec::new();
        if let Some(push_constant_def) = &definition.push_constant_def {
            push_constant_ranges.push(vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::ALL,
                offset: 0,
                size: push_constant_def.size.get(),
            });
        }

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&vk_descriptor_set_layouts[0..descriptor_set_layout_count])
            .push_constant_ranges(&push_constant_ranges)
            .build();

        let pipeline_layout = unsafe {
            device_context
                .device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)?
        };

        Ok(Self { pipeline_layout })
    }

    pub fn destroy(&self, device_context: &VulkanDeviceContext) {
        let device = device_context.device();

        unsafe {
            device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }

    pub fn vk_pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
}
