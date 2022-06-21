use std::sync::Arc;

use lgn_graphics_api::{
    DeviceContext, ExternalResourceType, Semaphore, SemaphoreDef, SemaphoreUsage, Texture,
};

use crate::{backends::nvenc::nv_encoder::NvEncoder, encoder_resource::EncoderResource};

#[derive(Clone)]
pub struct EncoderWorkItem {
    pub image: EncoderResource<Texture>,
    pub semaphore: EncoderResource<Semaphore>,
    pub semaphore_value: u64,
}

pub(crate) struct StreamEncoderInner {
    hw_encoder: Option<NvEncoder>,
}

#[derive(Clone)]
pub struct StreamEncoder {
    inner: Arc<StreamEncoderInner>,
}

impl StreamEncoder {
    pub fn new(force_software_encoding: bool) -> Self {
        Self {
            inner: Arc::new(StreamEncoderInner {
                hw_encoder: if force_software_encoding {
                    None
                } else {
                    NvEncoder::new()
                },
            }),
        }
    }

    pub fn new_external_image(
        &self,
        image: &Texture,
        device_context: &DeviceContext,
    ) -> EncoderResource<Texture> {
        let key = if let Some(encoder) = &self.inner.hw_encoder {
            encoder.register_external_image(device_context, image)
        } else {
            u64::MAX
        };
        EncoderResource::<Texture>::new(self, image, key)
    }

    pub fn new_external_semaphore(
        &self,
        device_context: &DeviceContext,
        mut semaphore_def: SemaphoreDef,
    ) -> EncoderResource<Semaphore> {
        semaphore_def.usage_flags |= SemaphoreUsage::EXPORT;
        let semaphore = device_context.create_semaphore(semaphore_def);
        let key = if let Some(encoder) = &self.inner.hw_encoder {
            encoder.register_external_semaphore(device_context, &semaphore)
        } else {
            u64::MAX
        };
        EncoderResource::<Semaphore>::new(self, &semaphore, key)
    }

    pub(crate) fn destroy_internal_resource(
        &self,
        resource_type: &ExternalResourceType,
        resource_key: u64,
    ) {
        if let Some(encoder) = &self.inner.hw_encoder {
            match resource_type {
                ExternalResourceType::Image => encoder.unregister_external_image(resource_key),
                ExternalResourceType::Semaphore => {
                    encoder.unregister_external_semaphore(resource_key);
                }
            }
        }
    }

    pub fn hw_encoder(&self) -> Option<NvEncoder> {
        self.inner.hw_encoder.clone()
    }
}

impl Default for StreamEncoder {
    fn default() -> Self {
        Self::new(false)
    }
}
