use lgn_graphics_api::{DeviceContext, Sampler, SamplerDef};
use parking_lot::RwLock;

use super::{PersistentDescriptorSetManager, SamplerSlot};

pub struct RenderSampler {
    sampler: Sampler,
    bindless_slot: SamplerSlot,
}

impl RenderSampler {
    #[allow(dead_code)]
    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    #[allow(dead_code)]
    pub fn bindless_slot(&self) -> SamplerSlot {
        self.bindless_slot
    }
}

pub struct SamplerManager {
    device_context: DeviceContext,
    persistent_descriptor_set_manager: PersistentDescriptorSetManager,
    samplers: RwLock<Vec<(u64, RenderSampler)>>,
}

impl SamplerManager {
    pub fn new(
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &PersistentDescriptorSetManager,
    ) -> Self {
        Self {
            device_context: device_context.clone(),
            persistent_descriptor_set_manager: persistent_descriptor_set_manager.clone(),
            samplers: RwLock::new(Vec::new()),
        }
    }

    pub fn get_slot(&self, sampler_definition: &SamplerDef) -> SamplerSlot {
        let samplers = self.samplers.read();
        if let Some(idx) = samplers
            .iter()
            .position(|s| s.0 == sampler_definition.get_hash())
            .map(|idx| idx as u32)
        {
            assert_eq!(
                sampler_definition,
                samplers[idx as usize].1.sampler.definition()
            );
            return samplers[idx as usize].1.bindless_slot;
        }
        drop(samplers);

        let mut samplers = self.samplers.write();
        let sampler = self.device_context.create_sampler(*sampler_definition);
        let bindless_slot = self
            .persistent_descriptor_set_manager
            .allocate_sampler_slot(&sampler);

        samplers.push((
            sampler_definition.get_hash(),
            RenderSampler {
                sampler,
                bindless_slot,
            },
        ));
        bindless_slot
    }
}
