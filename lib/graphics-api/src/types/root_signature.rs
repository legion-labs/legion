#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanRootSignature;
use crate::deferred_drop::Drc;
use crate::{DeviceContextDrc, GfxResult, PipelineType, RootSignatureDef};

// Not currently exposed
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct DynamicDescriptorIndex(pub(crate) u32);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct PushConstantIndex(pub(crate) u32);

pub struct RootSignature {
    device_context: DeviceContextDrc,
    definition: RootSignatureDef,

    #[cfg(feature = "vulkan")]
    pub(super) platform_root_signature: VulkanRootSignature,
}

impl Drop for RootSignature {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_root_signature
            .destroy(self.device_context.platform_device_context());
    }
}

#[derive(Clone)]
pub struct RootSignatureDrc {
    pub(super) inner: Drc<RootSignature>,
}

impl RootSignatureDrc {
    pub fn new(
        device_context: &DeviceContextDrc,
        definition: &RootSignatureDef,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_root_signature =
            VulkanRootSignature::new(device_context.platform_device_context(), definition)
                .map_err(|e| {
                    log::error!("Error creating platform root signature {:?}", e);
                    ash::vk::Result::ERROR_UNKNOWN
                })?;

        let inner = RootSignature {
            device_context: device_context.clone(),
            definition: definition.clone(),
            #[cfg(any(feature = "vulkan"))]
            platform_root_signature,
        };

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(inner),
        })
    }

    pub fn device_context(&self) -> &DeviceContextDrc {
        &self.inner.device_context
    }

    pub fn pipeline_type(&self) -> PipelineType {
        self.inner.definition.pipeline_type
    }

    pub fn definition(&self) -> &RootSignatureDef {
        &self.inner.definition
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_root_signature(&self) -> &VulkanRootSignature {
        &self.inner.platform_root_signature
    }
}
