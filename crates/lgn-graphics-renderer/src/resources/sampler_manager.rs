use lgn_graphics_api::{AddressMode, CompareOp, DeviceContext, FilterType, MipMapMode, SamplerDef};

use super::PersistentDescriptorSetManager;

const SAMPLER_ARRAY_SIZE: usize = 64; // When changing this number make sure to make a corresponding change to material_samplers in root.rn

pub struct SamplerManager {
    device_context: DeviceContext,

    samplers: Vec<(
        lgn_graphics_data::runtime::SamplerData,
        lgn_graphics_api::Sampler,
    )>,
}

impl SamplerManager {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
            samplers: Vec::new(),
        }
    }

    pub fn get_index(
        &mut self,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
        sampler_data: Option<&lgn_graphics_data::runtime::SamplerData>,
    ) -> u32 {
        if let Some(sampler_data) = sampler_data {
            if let Some(idx) = self
                .samplers
                .iter()
                .position(|s| s.0 == *sampler_data)
                .map(|idx| idx as u32)
            {
                return idx as u32;
            }

            assert!(self.samplers.len() < SAMPLER_ARRAY_SIZE);

            #[allow(clippy::match_same_arms)]
            let gpu_sampler = self.device_context.create_sampler(&SamplerDef {
                min_filter: match sampler_data.min_filter {
                    lgn_graphics_data::Filter::Nearest => FilterType::Nearest,
                    lgn_graphics_data::Filter::Linear => FilterType::Linear,
                    _ => FilterType::Linear,
                },
                mag_filter: match sampler_data.mag_filter {
                    lgn_graphics_data::Filter::Nearest => FilterType::Nearest,
                    lgn_graphics_data::Filter::Linear => FilterType::Linear,
                    _ => FilterType::Linear,
                },
                mip_map_mode: match sampler_data.mip_filter {
                    lgn_graphics_data::Filter::Nearest => MipMapMode::Nearest,
                    lgn_graphics_data::Filter::Linear => MipMapMode::Linear,
                    _ => MipMapMode::Linear,
                },
                address_mode_u: match sampler_data.wrap_u {
                    lgn_graphics_data::WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                    lgn_graphics_data::WrappingMode::MirroredRepeat => AddressMode::Mirror,
                    lgn_graphics_data::WrappingMode::Repeat => AddressMode::Repeat,
                    _ => AddressMode::Repeat,
                },
                address_mode_v: match sampler_data.wrap_v {
                    lgn_graphics_data::WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                    lgn_graphics_data::WrappingMode::MirroredRepeat => AddressMode::Mirror,
                    lgn_graphics_data::WrappingMode::Repeat => AddressMode::Repeat,
                    _ => AddressMode::Repeat,
                },
                address_mode_w: AddressMode::Repeat,
                mip_lod_bias: 0.0,
                max_anisotropy: 1.0,
                compare_op: CompareOp::LessOrEqual,
            });
            let idx = self.samplers.len() as u32;
            persistent_descriptor_set_manager.set_sampler(idx, &gpu_sampler);
            self.samplers.push((sampler_data.clone(), gpu_sampler));

            return idx;
        }
        0
    }
}
