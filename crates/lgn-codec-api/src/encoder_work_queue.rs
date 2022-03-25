use std::sync::{Arc, Mutex};

use lgn_graphics_api::{DeviceContext, ExternalResourceType, Semaphore, Texture};

use crate::encoder_resource::EncoderResource;

#[derive(Clone)]
pub struct EncoderWorkItem {
    pub image: EncoderResource<Texture>,
    pub semaphore: EncoderResource<Semaphore>,
}

#[derive(Default)]
pub(crate) struct EncoderWorkQueueInner {
    image_cleanup: Vec<u64>,
    semaphore_cleanup: Vec<u64>,
    shutting_down: bool,
}

#[derive(Clone)]
pub struct EncoderWorkQueue {
    inner: Arc<Mutex<EncoderWorkQueueInner>>,
}

impl EncoderWorkQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(EncoderWorkQueueInner::default())),
        }
    }

    pub fn new_external_image(
        &self,
        image: &Texture,
        device_context: &DeviceContext,
    ) -> EncoderResource<Texture> {
        EncoderResource::<Texture>::new(self, image, device_context)
    }

    pub fn new_external_semaphore(
        &self,
        device_context: &DeviceContext,
    ) -> EncoderResource<Semaphore> {
        EncoderResource::<Semaphore>::new(
            self,
            &device_context.create_semaphore(true),
            device_context,
        )
    }

    pub(crate) fn destroy_internal_resource(
        &self,
        resource_type: &ExternalResourceType,
        resource_key: u64,
    ) {
        let inner = &mut *self.inner.lock().unwrap();

        match resource_type {
            ExternalResourceType::Image => inner.image_cleanup.push(resource_key),
            ExternalResourceType::Semaphore => inner.semaphore_cleanup.push(resource_key),
        };
    }

    pub(crate) fn internal_image_for_cleanup(&self) -> Option<u64> {
        let inner = &mut *self.inner.lock().unwrap();

        inner.image_cleanup.pop()
    }

    pub(crate) fn internal_semaphore_for_cleanup(&self) -> Option<u64> {
        let inner = &mut *self.inner.lock().unwrap();

        inner.semaphore_cleanup.pop()
    }

    pub(crate) fn shutting_down(&self) -> bool {
        let inner = self.inner.lock().unwrap();

        inner.shutting_down
    }
}

impl Default for EncoderWorkQueue {
    fn default() -> Self {
        Self::new()
    }
}
