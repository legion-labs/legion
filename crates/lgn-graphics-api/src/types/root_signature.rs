use std::hash::{Hash, Hasher};

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanRootSignature;
use crate::deferred_drop::Drc;
use crate::{DeviceContext, GfxResult, RootSignatureDef};

// Not currently exposed
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct DynamicDescriptorIndex(pub(crate) u32);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct PushConstantIndex(pub(crate) u32);

#[derive(Debug)]
pub(crate) struct RootSignatureInner {
    device_context: DeviceContext,
    definition: RootSignatureDef,
    hash: u64,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_root_signature: VulkanRootSignature,
}

impl Drop for RootSignatureInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_root_signature.destroy(&self.device_context);
    }
}

#[derive(Debug, Clone)]
pub struct RootSignature {
    pub(crate) inner: Drc<RootSignatureInner>,
}

impl RootSignature {
    pub fn new(device_context: &DeviceContext, definition: &RootSignatureDef) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_root_signature = VulkanRootSignature::new(device_context, definition)
            .map_err(|e| {
                lgn_tracing::error!("Error creating platform root signature {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;

        let mut hasher = fnv::FnvHasher::default();
        definition.hash(&mut hasher);

        let inner = RootSignatureInner {
            device_context: device_context.clone(),
            definition: definition.clone(),
            hash: hasher.finish(),
            #[cfg(any(feature = "vulkan"))]
            platform_root_signature,
        };

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(inner),
        })
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn definition(&self) -> &RootSignatureDef {
        &self.inner.definition
    }
}

impl PartialEq for RootSignature {
    fn eq(&self, other: &Self) -> bool {
        self.inner.hash == other.inner.hash
    }
}
