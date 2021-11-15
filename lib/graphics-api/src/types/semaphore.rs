use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanSemaphore;
use crate::{DeviceContextDrc, GfxResult};

pub struct Semaphore {
    device_context: DeviceContextDrc,

    // Set to true when an operation is scheduled to signal this semaphore
    // Cleared when an operation is scheduled to consume this semaphore
    signal_available: AtomicBool,

    #[cfg(feature = "vulkan")]
    platform_semaphore: VulkanSemaphore,
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_semaphore.destroy(&self.device_context);
    }
}

impl Semaphore {
    pub fn new(device_context: &DeviceContextDrc) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_semaphore = VulkanSemaphore::new(device_context).map_err(|e| {
            log::error!("Error creating semaphore {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        Ok(Self {
            device_context: device_context.clone(),
            signal_available: AtomicBool::new(false),
            #[cfg(any(feature = "vulkan"))]
            platform_semaphore,
        })
    }

    pub fn signal_available(&self) -> bool {
        self.signal_available.load(Ordering::Relaxed)
    }

    #[cfg(any(feature = "vulkan"))]
    pub fn set_signal_available(&self, available: bool) {
        self.signal_available.store(available, Ordering::Relaxed);
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_semaphore(&self) -> &VulkanSemaphore {
        &self.platform_semaphore
    }
}
