use lgn_graphics_api::{DeviceContext, SamplerDef};
use parking_lot::RwLock;

use super::PersistentDescriptorSetManager;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct SamplerId(u32);

impl SamplerId {
    pub fn as_index(self) -> u32 {
        self.0
    }
}

const DEFAULT_SAMPLER_ID: SamplerId = SamplerId(0);

pub struct SamplerManager {
    device_context: DeviceContext,

    samplers: RwLock<Vec<(u64, lgn_graphics_api::Sampler)>>,
    uploaded: usize,
}

impl SamplerManager {
    pub fn new(
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Self {
        let mut sampler_manager = Self {
            device_context: device_context.clone(),
            samplers: RwLock::new(Vec::new()),
            uploaded: 0,
        };
        let idx = sampler_manager.get_index(&SamplerDef::default());
        assert_eq!(idx, DEFAULT_SAMPLER_ID);
        sampler_manager.upload(persistent_descriptor_set_manager);
        sampler_manager
    }

    pub fn get_index(&self, sampler_definition: &SamplerDef) -> SamplerId {
        let samplers = self.samplers.read();
        if let Some(idx) = samplers
            .iter()
            .position(|s| s.0 == sampler_definition.get_hash())
            .map(|idx| idx as u32)
        {
            assert_eq!(sampler_definition, samplers[idx as usize].1.definition());
            return SamplerId(idx);
        }
        drop(samplers);

        let mut samplers = self.samplers.write();
        let sampler_id = SamplerId(samplers.len() as u32);
        samplers.push((
            sampler_definition.get_hash(),
            self.device_context.create_sampler(*sampler_definition),
        ));
        sampler_id
    }

    pub fn upload(
        &mut self,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) {
        let samplers = self.samplers.read();
        for idx in self.uploaded..samplers.len() {
            persistent_descriptor_set_manager.set_sampler(idx as u32, &samplers[idx].1);
        }
        self.uploaded = samplers.len();
    }

    pub fn get_default_sampler_index() -> SamplerId {
        DEFAULT_SAMPLER_ID
    }
}
