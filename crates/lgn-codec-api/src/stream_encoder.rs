use std::sync::Arc;

use lgn_graphics_api::{DeviceContext, ExternalResourceType, Semaphore, Texture};

use crate::{backends::nvenc::nv_encoder::NvEncoder, encoder_resource::EncoderResource};

#[derive(Clone)]
pub struct EncoderWorkItem {
    pub image: EncoderResource<Texture>,
    pub semaphore: EncoderResource<Semaphore>,
}

pub(crate) struct StreamEncoderInner {
    enable_hw_encoding: bool,
    hw_encoder: Option<NvEncoder>,
}

#[derive(Clone)]
pub struct StreamEncoder {
    inner: Arc<StreamEncoderInner>,
}

impl StreamEncoder {
    pub fn new(enable_hw_encoding: bool) -> Self {
        Self {
            inner: Arc::new(StreamEncoderInner {
                enable_hw_encoding,
                hw_encoder: if enable_hw_encoding {
                    NvEncoder::new()
                } else {
                    None
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
    ) -> EncoderResource<Semaphore> {
        let semaphore = device_context.create_semaphore(true);
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
        if self.inner.enable_hw_encoding {
            self.inner.hw_encoder.clone()
        } else {
            None
        }
    }
}

impl Default for StreamEncoder {
    fn default() -> Self {
        Self::new(false)
    }
}
