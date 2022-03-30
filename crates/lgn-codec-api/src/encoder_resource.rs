use std::sync::Arc;

use lgn_graphics_api::ExternalResource;

use crate::stream_encoder::StreamEncoder;

pub struct EncoderResourceInner<T: ExternalResource<T>> {
    external_resource: T,
    internal_resource: u64,
    work_queue: StreamEncoder,
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
    inner: Arc<EncoderResourceInner<T>>,
}

impl<T: ExternalResource<T>> EncoderResource<T> {
    pub(crate) fn new(
        work_queue: &StreamEncoder,
        external_resource: &T,
        internal_resource: u64,
    ) -> Self {
        Self {
            inner: Arc::new(EncoderResourceInner {
                external_resource: external_resource.clone_resource(),
                internal_resource,
                work_queue: work_queue.clone(),
            }),
        }
    }

    pub fn external_resource(&self) -> T {
        self.inner.external_resource.clone_resource()
    }

    pub(crate) fn internal_resource(&self) -> u64 {
        self.inner.internal_resource
    }
}
