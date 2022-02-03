use crate::backends::BackendSampler;
use crate::deferred_drop::Drc;
use crate::{DeviceContext, GfxResult, SamplerDef};

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
    pub fn new(device_context: &DeviceContext, sampler_def: &SamplerDef) -> GfxResult<Self> {
        let platform_sampler = BackendSampler::new(device_context, sampler_def)?;
        let inner = SamplerInner {
            device_context: device_context.clone(),
            backend_sampler: platform_sampler,
        };

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(inner),
        })
    }
}
