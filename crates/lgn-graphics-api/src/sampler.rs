use lgn_utils::decimal::DecimalF32;
use std::hash::{Hash, Hasher};

use crate::backends::BackendSampler;
use crate::deferred_drop::Drc;
use crate::{AddressMode, CompareOp, DeviceContext, FilterType, MipMapMode};

/// Used to create a `Sampler`
#[derive(Debug, Clone, Default)]
pub struct SamplerDef {
    pub min_filter: FilterType,
    pub mag_filter: FilterType,
    pub mip_map_mode: MipMapMode,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub mip_lod_bias: f32,
    pub max_anisotropy: f32,
    pub compare_op: CompareOp,
    //NOTE: Custom hash impl, don't forget to add changes there too!
}

impl Eq for SamplerDef {}
impl PartialEq for SamplerDef {
    fn eq(&self, other: &Self) -> bool {
        self.min_filter == other.min_filter
            && self.mag_filter == other.mag_filter
            && self.mip_map_mode == other.mip_map_mode
            && self.address_mode_u == other.address_mode_u
            && self.address_mode_v == other.address_mode_v
            && self.address_mode_w == other.address_mode_w
            && DecimalF32(self.mip_lod_bias) == DecimalF32(other.mip_lod_bias)
            && DecimalF32(self.max_anisotropy) == DecimalF32(other.max_anisotropy)
            && self.compare_op == other.compare_op
    }
}

impl Hash for SamplerDef {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        self.min_filter.hash(&mut state);
        self.mag_filter.hash(&mut state);
        self.mip_map_mode.hash(&mut state);
        self.address_mode_u.hash(&mut state);
        self.address_mode_v.hash(&mut state);
        self.address_mode_w.hash(&mut state);
        DecimalF32(self.mip_lod_bias).hash(&mut state);
        DecimalF32(self.max_anisotropy).hash(&mut state);
        self.compare_op.hash(&mut state);
    }
}

pub(crate) struct SamplerInner {
    device_context: DeviceContext,
    pub(crate) backend_sampler: BackendSampler,
}

impl Drop for SamplerInner {
    fn drop(&mut self) {
        self.backend_sampler.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct Sampler {
    pub(crate) inner: Drc<SamplerInner>,
}

impl Sampler {
    pub fn new(device_context: &DeviceContext, sampler_def: &SamplerDef) -> Self {
        let platform_sampler = BackendSampler::new(device_context, sampler_def);
        let inner = SamplerInner {
            device_context: device_context.clone(),
            backend_sampler: platform_sampler,
        };

        Self {
            inner: device_context.deferred_dropper().new_drc(inner),
        }
    }
}
