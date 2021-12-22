use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanSemaphore;
use crate::{deferred_drop::Drc, DeviceContext};

pub(crate) struct SemaphoreInner {
    device_context: DeviceContext,

    // Set to true when an operation is scheduled to signal this semaphore
    // Cleared when an operation is scheduled to consume this semaphore
    signal_available: AtomicBool,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_semaphore: VulkanSemaphore,
}

pub struct Semaphore {
    pub(crate) inner: Drc<SemaphoreInner>,
}

impl Drop for SemaphoreInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_semaphore.destroy(&self.device_context);
    }
}

impl Semaphore {
    pub fn new(device_context: &DeviceContext) -> Self {
        #[cfg(feature = "vulkan")]
        let platform_semaphore = VulkanSemaphore::new(device_context);

        Self {
            inner: device_context.deferred_dropper().new_drc(SemaphoreInner {
                device_context: device_context.clone(),
                signal_available: AtomicBool::new(false),
                #[cfg(any(feature = "vulkan"))]
                platform_semaphore,
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
