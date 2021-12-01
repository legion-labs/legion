#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanSemaphore;
use crate::DeviceContext;

#[derive(Clone, Copy)]
pub struct Semaphore {
    #[cfg(feature = "vulkan")]
    platform_semaphore: VulkanSemaphore,
}

impl Semaphore {
    pub fn new(device_context: &DeviceContext) -> Self {
        #[cfg(feature = "vulkan")]
        let platform_semaphore = VulkanSemaphore::new(device_context);

        Self {
            #[cfg(any(feature = "vulkan"))]
            platform_semaphore,
        }
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_semaphore.destroy(device_context);
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_semaphore(&self) -> &VulkanSemaphore {
        &self.platform_semaphore
    }
}
