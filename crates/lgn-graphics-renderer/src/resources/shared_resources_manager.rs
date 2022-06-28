use std::sync::Arc;

use lgn_graphics_api::{DeviceContext, Sampler, SamplerDef};

use super::{PersistentDescriptorSetManager, SamplerSlot};

#[derive(Clone)]
struct DefaultSampler {
    _sampler: Sampler,
    bindless_slot: SamplerSlot,
}

struct Inner {
    default_sampler: DefaultSampler,
}

#[derive(Clone)]
pub struct SharedResourcesManager {
    inner: Arc<Inner>,
}

impl SharedResourcesManager {
    pub fn new(
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Self {
        let default_sampler =
            Self::create_default_sampler(device_context, persistent_descriptor_set_manager);

        Self {
            inner: Arc::new(Inner { default_sampler }),
        }
    }

    pub fn default_sampler_slot(&self) -> SamplerSlot {
        self.inner.default_sampler.bindless_slot
    }

    fn create_default_sampler(
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> DefaultSampler {
        let sampler = device_context.create_sampler(SamplerDef::default());
        let bindless_slot = persistent_descriptor_set_manager.allocate_sampler_slot(&sampler);
        DefaultSampler {
            _sampler: sampler,
            bindless_slot,
        }
    }
}
