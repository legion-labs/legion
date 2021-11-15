#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanSampler;
use crate::deferred_drop::Drc;
use crate::{DeviceContextDrc, GfxResult, SamplerDef};

pub struct Sampler {
    device_context: DeviceContextDrc,

    #[cfg(feature = "vulkan")]
    platform_sampler: VulkanSampler,
}

impl Drop for Sampler {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_sampler.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct SamplerDrc {
    inner: Drc<Sampler>,
}

impl SamplerDrc {
    pub fn new(device_context: &DeviceContextDrc, sampler_def: &SamplerDef) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_sampler = VulkanSampler::new(device_context, sampler_def).map_err(|e| {
            log::error!("Error creating platform sampler {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        let inner = Sampler {
            device_context: device_context.clone(),
            #[cfg(any(feature = "vulkan"))]
            platform_sampler,
        };

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(inner),
        })
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_sampler(&self) -> &VulkanSampler {
        &self.inner.platform_sampler
    }
}
