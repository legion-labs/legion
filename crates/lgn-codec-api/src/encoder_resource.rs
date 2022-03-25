use std::{
    ffi::c_void,
    sync::{Arc, Mutex},
};

use lgn_graphics_api::{DeviceContext, ExternalResource};

use crate::encoder_work_queue::EncoderWorkQueue;

pub struct EncoderResourceInner<T: ExternalResource<T>> {
    device_context: DeviceContext,
    external_resource: T,
    internal_resource: u64,
    work_queue: EncoderWorkQueue,
}

impl<T: ExternalResource<T>> Drop for EncoderResourceInner<T> {
    fn drop(&mut self) {
        if self.internal_resource != u64::MAX {
            self.work_queue
                .destroy_internal_resource(&T::external_resource_type(), self.internal_resource);
        }
    }
}

#[derive(Clone)]
pub struct EncoderResource<T: ExternalResource<T>> {
    inner: Arc<Mutex<EncoderResourceInner<T>>>,
}

impl<T: ExternalResource<T>> EncoderResource<T> {
    pub(crate) fn new(
        work_queue: &EncoderWorkQueue,
        external_resource: &T,
        device_context: &DeviceContext,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EncoderResourceInner {
                device_context: device_context.clone(),
                external_resource: external_resource.clone_resource(),
                internal_resource: u64::MAX,
                work_queue: work_queue.clone(),
            })),
        }
    }

    pub fn external_resource(&self) -> T {
        let inner = &mut *self.inner.lock().unwrap();
        inner.external_resource.clone_resource()
    }

    pub(crate) fn external_resource_handle(&self) -> *mut c_void {
        let inner = &mut *self.inner.lock().unwrap();
        inner
            .external_resource
            .external_resource_handle(&inner.device_context)
    }

    pub(crate) fn internal_resource(&self) -> u64 {
        let inner = self.inner.lock().unwrap();
        inner.internal_resource
    }

    pub(crate) fn update_internal_resource(&self, internal_resource: u64) {
        let inner = &mut *self.inner.lock().unwrap();
        inner.internal_resource = internal_resource;
    }
}
