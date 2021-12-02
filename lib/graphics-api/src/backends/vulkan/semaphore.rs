use crate::{DeviceContext, GfxResult, Semaphore};

pub(crate) struct VulkanSemaphore {
    vk_semaphore: ash::vk::Semaphore,
}

impl VulkanSemaphore {
    pub fn new(device_context: &DeviceContext) -> GfxResult<Self> {
        let create_info =
            ash::vk::SemaphoreCreateInfo::builder().flags(ash::vk::SemaphoreCreateFlags::empty());

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
}

impl Semaphore {
    pub fn vk_semaphore(&self) -> ash::vk::Semaphore {
        self.inner.platform_semaphore.vk_semaphore
    }
}
