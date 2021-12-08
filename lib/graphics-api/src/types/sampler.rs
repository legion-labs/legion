#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanSampler;
use crate::deferred_drop::Drc;
use crate::{DeviceContext, GfxResult, SamplerDef};

pub(crate) struct SamplerInner {
    device_context: DeviceContext,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_sampler: VulkanSampler,
}

impl Drop for SamplerInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_sampler.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct Sampler {
    pub(crate) inner: Drc<SamplerInner>,
}

impl Sampler {
    pub fn new(device_context: &DeviceContext, sampler_def: &SamplerDef) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_sampler = VulkanSampler::new(device_context, sampler_def).map_err(|e| {
            log::error!("Error creating platform sampler {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        let inner = SamplerInner {
            device_context: device_context.clone(),
            #[cfg(any(feature = "vulkan"))]
            platform_sampler,
        };

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(inner),
        })
    }
}
