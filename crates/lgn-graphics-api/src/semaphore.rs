use std::sync::atomic::{AtomicBool, Ordering};

use crate::ExternalResourceHandle;
use crate::{
    backends::BackendSemaphore, deferred_drop::Drc, DeviceContext, ExternalResource,
    ExternalResourceType,
};

bitflags::bitflags! {
    pub struct SemaphoreUsage: u16 {
        const TIMELINE = 0x0001;
        const EXPORT = 0x0002;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SemaphoreDef {
    pub usage_flags: SemaphoreUsage,
    pub initial_value: u64,
}

impl Default for SemaphoreDef {
    fn default() -> Self {
        Self {
            usage_flags: SemaphoreUsage::empty(),
            initial_value: 0,
        }
    }
}

pub(crate) struct SemaphoreInner {
    semaphore_def: SemaphoreDef,
    device_context: DeviceContext,

    // Set to true when an operation is scheduled to signal this semaphore
    // Cleared when an operation is scheduled to consume this semaphore
    signal_available: AtomicBool,
    pub(crate) backend_semaphore: BackendSemaphore,
}

#[derive(Clone)]
pub struct Semaphore {
    pub(crate) inner: Drc<SemaphoreInner>,
}

impl Drop for SemaphoreInner {
    fn drop(&mut self) {
        self.backend_semaphore.destroy(&self.device_context);
    }
}

impl Semaphore {
    pub fn new(device_context: &DeviceContext, semaphore_def: SemaphoreDef) -> Self {
        let platform_semaphore = BackendSemaphore::new(device_context, semaphore_def);

        Self {
            inner: device_context.deferred_dropper().new_drc(SemaphoreInner {
                semaphore_def,
                device_context: device_context.clone(),
                signal_available: AtomicBool::new(false),
                backend_semaphore: platform_semaphore,
            }),
        }
    }

    pub fn signal_available(&self) -> bool {
        self.inner.signal_available.load(Ordering::Relaxed)
    }

    pub fn set_signal_available(&self, available: bool) {
        self.inner
            .signal_available
            .store(available, Ordering::Relaxed);
    }

    pub fn definition(&self) -> SemaphoreDef {
        self.inner.semaphore_def
    }
}

impl ExternalResource<Self> for Semaphore {
    fn clone_resource(&self) -> Self {
        self.clone()
    }

    fn external_resource_type() -> ExternalResourceType {
        ExternalResourceType::Semaphore
    }

    fn external_resource_handle(&self, device_context: &DeviceContext) -> ExternalResourceHandle {
        self.inner
            .backend_semaphore
            .external_semaphore_handle(device_context)
    }
}
