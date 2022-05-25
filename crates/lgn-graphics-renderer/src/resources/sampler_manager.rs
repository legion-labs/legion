use lgn_app::App;
use lgn_ecs::{prelude::ResMut, schedule::SystemSet};
use lgn_graphics_api::{DeviceContext, SamplerDef};

use crate::{labels::RenderStage, ResourceStageLabel};

use super::PersistentDescriptorSetManager;

const SAMPLER_ARRAY_SIZE: usize = 64; // When changing this number make sure to make a corresponding change to material_samplers in root.rn

pub struct SamplerManager {
    device_context: DeviceContext,

    samplers: Vec<(SamplerDef, lgn_graphics_api::Sampler)>,
    scheduled_upload: Vec<u32>,
}

impl SamplerManager {
    pub fn new(
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Self {
        let mut sampler_manager = Self {
            device_context: device_context.clone(),
            samplers: Vec::new(),
            scheduled_upload: Vec::new(),
        };
        sampler_manager.get_index(&SamplerDef::default());
        sampler_manager.upload(persistent_descriptor_set_manager);
        sampler_manager
    }

    pub fn init_ecs(app: &mut App) {
        app.add_system_set_to_stage(
            RenderStage::Resource,
            SystemSet::new()
                .with_system(upload_sampler_data)
                .label(ResourceStageLabel::Sampler)
                .after(ResourceStageLabel::Material),
        );
    }

    pub fn get_index(&mut self, sampler_definition: &SamplerDef) -> u32 {
        if let Some(idx) = self
            .samplers
            .iter()
            .position(|s| s.0 == *sampler_definition)
            .map(|idx| idx as u32)
        {
            return idx as u32;
        }
        assert!(self.samplers.len() < SAMPLER_ARRAY_SIZE);
        let idx = self.samplers.len() as u32;
        self.samplers.push((
            sampler_definition.clone(),
            self.device_context.create_sampler(sampler_definition),
        ));
        self.scheduled_upload.push(idx);
        idx
    }

    pub fn upload(
        &mut self,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) {
        self.scheduled_upload.iter().for_each(|idx| {
            persistent_descriptor_set_manager.set_sampler(*idx, &self.samplers[*idx as usize].1);
        });
        self.scheduled_upload.clear();
    }

    pub fn get_default_sampler_index() -> u32 {
        0
    }
}

pub(crate) fn upload_sampler_data(
    mut sampler_manager: ResMut<'_, SamplerManager>,
    mut persistent_descriptor_set_manager: ResMut<'_, PersistentDescriptorSetManager>,
) {
    sampler_manager.upload(&mut persistent_descriptor_set_manager);
}
