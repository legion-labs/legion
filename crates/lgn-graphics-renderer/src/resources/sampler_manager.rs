use lgn_graphics_api::{DeviceContext, SamplerDef};
use parking_lot::Mutex;

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

    samplers: Mutex<Vec<(SamplerDef, lgn_graphics_api::Sampler)>>,
    scheduled_upload: Mutex<Vec<SamplerId>>,
}

impl SamplerManager {
    pub fn new(
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Self {
        let sampler_manager = Self {
            device_context: device_context.clone(),
            samplers: Mutex::new(Vec::new()),
            scheduled_upload: Mutex::new(Vec::new()),
        };
        let idx = sampler_manager.get_index(&SamplerDef::default());
        assert_eq!(idx, DEFAULT_SAMPLER_ID);
        sampler_manager.upload(persistent_descriptor_set_manager);
        sampler_manager
    }

    pub fn get_index(&self, sampler_definition: &SamplerDef) -> SamplerId {
        let mut samplers = self.samplers.lock();
        let mut scheduled_upload = self.scheduled_upload.lock();
        if let Some(idx) = samplers
            .iter()
            .position(|s| s.0 == *sampler_definition)
            .map(|idx| idx as u32)
        {
            return SamplerId(idx);
        }
        let sampler_id = SamplerId(samplers.to_owned().len() as u32);
        samplers.push((
            *sampler_definition,
            self.device_context.create_sampler(*sampler_definition),
        ));
        scheduled_upload.push(sampler_id);
        sampler_id
    }

    pub fn upload(&self, persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager) {
        let samplers = self.samplers.lock();
        let mut scheduled_upload = self.scheduled_upload.lock();
        scheduled_upload.iter().for_each(|idx| {
            persistent_descriptor_set_manager.set_sampler(idx.0, &samplers[idx.0 as usize].1);
        });
        scheduled_upload.clear();
    }

    pub fn get_default_sampler_index() -> SamplerId {
        DEFAULT_SAMPLER_ID
    }
}
