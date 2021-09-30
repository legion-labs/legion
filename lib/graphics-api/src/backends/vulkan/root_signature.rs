use super::{VulkanApi, VulkanDescriptorSetLayout, VulkanDeviceContext};
use crate::{GfxResult, PipelineType, RootSignature, RootSignatureDef, MAX_DESCRIPTOR_SET_LAYOUTS};

use ash::vk;
use std::sync::Arc;

// Not currently exposed
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct DynamicDescriptorIndex(pub(crate) u32);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct PushConstantIndex(pub(crate) u32);

#[derive(Debug)]
pub(crate) struct RootSignatureVulkanInner {
    pub(crate) device_context: VulkanDeviceContext,
    pub(crate) pipeline_type: PipelineType,
    pub(crate) layouts: [Option<VulkanDescriptorSetLayout>; MAX_DESCRIPTOR_SET_LAYOUTS],
    pub(crate) pipeline_layout: vk::PipelineLayout,
}

impl Drop for RootSignatureVulkanInner {
    fn drop(&mut self) {
        let device = self.device_context.device();

        unsafe {
            device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

#[derive(Clone, Debug)]
pub struct VulkanRootSignature {
    pub(crate) inner: Arc<RootSignatureVulkanInner>,
}

impl VulkanRootSignature {
    pub fn device_context(&self) -> &VulkanDeviceContext {
        &self.inner.device_context
    }

    pub fn vk_pipeline_layout(&self) -> vk::PipelineLayout {
        self.inner.pipeline_layout
    }

    pub fn new(
        device_context: &VulkanDeviceContext,
        root_signature_def: &RootSignatureDef<VulkanApi>,
    ) -> GfxResult<Self> {
        log::trace!("Create VulkanRootSignature");

        //
        // Create pipeline layout
        //
        let mut vk_descriptor_set_layouts =
            [vk::DescriptorSetLayout::null(); MAX_DESCRIPTOR_SET_LAYOUTS];

        let mut descriptor_set_layout_count = 0;
        for layout in root_signature_def
            .descriptor_set_layouts
            .iter()
            .filter_map(|x| x.as_ref())
        {
            vk_descriptor_set_layouts[descriptor_set_layout_count] = layout.vk_layout();
            descriptor_set_layout_count += 1;
        }

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&vk_descriptor_set_layouts[0..descriptor_set_layout_count]);

        let pipeline_layout = unsafe {
            device_context
                .device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)?
        };

        let inner = RootSignatureVulkanInner {
            device_context: device_context.clone(),
            pipeline_type: root_signature_def.pipeline_type,
            layouts: root_signature_def.descriptor_set_layouts.clone(),
            pipeline_layout,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

impl RootSignature<VulkanApi> for VulkanRootSignature {
    fn pipeline_type(&self) -> PipelineType {
        self.inner.pipeline_type
    }
}
