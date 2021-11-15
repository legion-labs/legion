use ash::vk;

use super::VulkanQueue;
use crate::{CommandPoolDef, DeviceContextDrc, GfxResult};

pub(crate) struct VulkanCommandPool {
    vk_command_pool: vk::CommandPool,
}

impl VulkanCommandPool {
    pub(crate) fn new(
        devie_context: &DeviceContextDrc,
        queue: &VulkanQueue,
        command_pool_def: &CommandPoolDef,
    ) -> GfxResult<Self> {
        let queue_family_index = queue.vk_queue().queue_family_index();
        log::trace!(
            "Creating command pool on queue family index {:?}",
            queue_family_index
        );

        let mut command_pool_create_flags = vk::CommandPoolCreateFlags::empty();
        if command_pool_def.transient {
            command_pool_create_flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }

        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(command_pool_create_flags)
            .queue_family_index(queue_family_index);

        let vk_command_pool = unsafe {
            devie_context
                .platform_device()
                .create_command_pool(&pool_create_info, None)?
        };

        Ok(Self { vk_command_pool })
    }

    pub fn destroy(&self, device_context: &DeviceContextDrc) {
        unsafe {
            device_context
                .platform_device()
                .destroy_command_pool(self.vk_command_pool, None);
        }
    }

    pub fn reset_command_pool(&self, device_context: &DeviceContextDrc) -> GfxResult<()> {
        unsafe {
            device_context
                .platform_device()
                .reset_command_pool(self.vk_command_pool, vk::CommandPoolResetFlags::empty())?;
        }
        Ok(())
    }

    pub fn vk_command_pool(&self) -> vk::CommandPool {
        self.vk_command_pool
    }
}
