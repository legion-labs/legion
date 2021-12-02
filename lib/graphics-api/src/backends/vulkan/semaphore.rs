use ash::vk;

use crate::{DeviceContext, GfxResult};

pub(crate) struct VulkanSemaphore {
    vk_semaphore: vk::Semaphore,
}

impl VulkanSemaphore {
    pub fn new(device_context: &DeviceContext) -> GfxResult<Self> {
        let create_info =
            vk::SemaphoreCreateInfo::builder().flags(vk::SemaphoreCreateFlags::empty());

        let vk_semaphore = unsafe {
            device_context
                .vk_device()
                .create_semaphore(&*create_info, None)?
        };

        Ok(Self { vk_semaphore })
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .destroy_semaphore(self.vk_semaphore, None);
        }
    }

    pub fn vk_semaphore(&self) -> vk::Semaphore {
        self.vk_semaphore
    }
}
