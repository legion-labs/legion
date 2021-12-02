use ash::vk;

use crate::{CompareOp, DeviceContext, GfxResult, MipMapMode, SamplerDef};

pub(crate) struct VulkanSampler {
    sampler: vk::Sampler,
}

impl std::fmt::Debug for VulkanSampler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanSampler")
            .field("sampler", &self.sampler)
            .finish()
    }
}

impl VulkanSampler {
    pub fn new(device_context: &DeviceContext, sampler_def: &SamplerDef) -> GfxResult<Self> {
        let max_lod = if sampler_def.mip_map_mode == MipMapMode::Linear {
            f32::MAX
        } else {
            0.0
        };

        let sampler_create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(sampler_def.mag_filter.into())
            .min_filter(sampler_def.min_filter.into())
            .mipmap_mode(sampler_def.mip_map_mode.into())
            .address_mode_u(sampler_def.address_mode_u.into())
            .address_mode_v(sampler_def.address_mode_v.into())
            .address_mode_w(sampler_def.address_mode_w.into())
            .mip_lod_bias(sampler_def.mip_lod_bias)
            .anisotropy_enable(sampler_def.max_anisotropy > 0.0)
            .max_anisotropy(sampler_def.max_anisotropy)
            .compare_enable(sampler_def.compare_op != CompareOp::Never)
            .compare_op(sampler_def.compare_op.into())
            .min_lod(sampler_def.mip_lod_bias)
            .max_lod(max_lod)
            .border_color(vk::BorderColor::FLOAT_TRANSPARENT_BLACK)
            .unnormalized_coordinates(false);

        let sampler = unsafe {
            device_context
                .vk_device()
                .create_sampler(&*sampler_create_info, None)?
        };

        Ok(Self { sampler })
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .destroy_sampler(self.sampler, None);
        }
    }

    pub fn vk_sampler(&self) -> vk::Sampler {
        self.sampler
    }
}
