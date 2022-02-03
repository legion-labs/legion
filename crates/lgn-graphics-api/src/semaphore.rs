use std::sync::atomic::{AtomicBool, Ordering};

use crate::{backends::BackendSemaphore, deferred_drop::Drc, DeviceContext};

pub(crate) struct SemaphoreInner {
    device_context: DeviceContext,

    // Set to true when an operation is scheduled to signal this semaphore
    // Cleared when an operation is scheduled to consume this semaphore
    signal_available: AtomicBool,
    pub(crate) backend_semaphore: BackendSemaphore,
}

pub struct Semaphore {
    pub(crate) inner: Drc<SemaphoreInner>,
}

impl Drop for SemaphoreInner {
    fn drop(&mut self) {
        self.backend_semaphore.destroy(&self.device_context);
    }
}

impl Semaphore {
    pub fn new(device_context: &DeviceContext) -> Self {
        let platform_semaphore = BackendSemaphore::new(device_context);

        Self {
            inner: device_context.deferred_dropper().new_drc(SemaphoreInner {
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
}
