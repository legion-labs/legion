use lgn_graphics_api::{AddressMode, CompareOp, FilterType, MipMapMode, SamplerDef};
use lgn_graphics_data::runtime::Sampler;

use super::PersistentDescriptorSetManager;

const SAMPLER_ARRAY_SIZE: usize = 64;
pub struct SamplerManager {
    samplers: [Sampler; SAMPLER_ARRAY_SIZE],
    num: u32,
}

impl SamplerManager {
    pub fn new() -> SamplerManager {
        SamplerManager {
            samplers: [Sampler::default(); SAMPLER_ARRAY_SIZE],
            num: 0,
        }
    }

    pub fn upload_sampler_data(
        &mut self,
        persistent_descriptor_set_manager: &PersistentDescriptorSetManager,
        sampler: &Sampler,
    ) {
        if self.find_sampler(sampler).is_none() {
            persistent_descriptor_set_manager.set_sampler(
                self.num,
                SamplerDef {
                    min_filter: match sampler.min_filter {
                        lgn_graphics_data::MinFilter::Nearest
                        | lgn_graphics_data::MinFilter::NearestMipmapNearest
                        | lgn_graphics_data::MinFilter::NearestMipmapLinear => FilterType::Nearest,
                        lgn_graphics_data::MinFilter::Linear
                        | lgn_graphics_data::MinFilter::LinearMipmapNearest
                        | lgn_graphics_data::MinFilter::LinearMipmapLinear => FilterType::Linear,
                        _ => FilterType::Linear,
                    },
                    mag_filter: match sampler.mag_filter {
                        lgn_graphics_data::MagFilter::Nearest => FilterType::Nearest,
                        lgn_graphics_data::MagFilter::Linear => FilterType::Linear,
                        _ => todo!(),
                    },
                    mip_map_mode: match sampler.min_filter {
                        lgn_graphics_data::MinFilter::Nearest => MipMapMode::Linear,
                        lgn_graphics_data::MinFilter::Linear => MipMapMode::Linear,
                        lgn_graphics_data::MinFilter::NearestMipmapLinear => MipMapMode::Linear,
                        lgn_graphics_data::MinFilter::LinearMipmapLinear => MipMapMode::Linear,
                        lgn_graphics_data::MinFilter::NearestMipmapNearest => MipMapMode::Nearest,
                        lgn_graphics_data::MinFilter::LinearMipmapNearest => MipMapMode::Nearest,
                        _ => todo!(),
                    },
                    address_mode_u: match sampler.wrap_u {
                        lgn_graphics_data::WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                        lgn_graphics_data::WrappingMode::MirroredRepeat => AddressMode::Mirror,
                        lgn_graphics_data::WrappingMode::Repeat => AddressMode::Repeat,
                        _ => todo!(),
                    },
                    address_mode_v: match sampler.wrap_v {
                        lgn_graphics_data::WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                        lgn_graphics_data::WrappingMode::MirroredRepeat => AddressMode::Mirror,
                        lgn_graphics_data::WrappingMode::Repeat => AddressMode::Repeat,
                        _ => todo!(),
                    },
                    address_mode_w: AddressMode::Repeat,
                    mip_lod_bias: 0.0,
                    max_anisotropy: 1.0,
                    compare_op: CompareOp::LessOrEqual,
                },
            );
        }
    }

    pub fn get_index(&mut self, sampler: Option<&Sampler>) -> u32 {
        if let Some(sampler) = sampler {
            if let Some(idx) = self.find_sampler(sampler) {
                return idx as u32;
            }
            self.add_sampler(sampler);
            return self.num;
        }
        return SAMPLER_ARRAY_SIZE as u32;
    }

    fn add_sampler(&self, sampler: &Sampler) {
        assert!(self.num < SAMPLER_ARRAY_SIZE as u32);
        self.samplers[self.num as usize] = *sampler;
        self.num += 1;
    }

    fn find_sampler(&self, sampler: &Sampler) -> Option<u32> {
        self.samplers
            .iter()
            .position(|s| s == sampler)
            .map(|idx| idx as u32)
    }
}
