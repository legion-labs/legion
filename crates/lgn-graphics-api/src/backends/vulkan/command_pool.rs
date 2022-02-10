use lgn_tracing::trace;

use crate::{CommandPool, CommandPoolDef, DeviceContext, GfxResult, Queue};

pub(crate) struct VulkanCommandPool {
    vk_command_pool: ash::vk::CommandPool,
}

impl VulkanCommandPool {
    pub(crate) fn new(
        device_context: &DeviceContext,
        queue: &Queue,
        command_pool_def: &CommandPoolDef,
    ) -> GfxResult<Self> {
        let queue_family_index = queue.family_index();
        trace!(
            "Creating command pool on queue family index {:?}",
            queue_family_index
        );

        let mut command_pool_create_flags = ash::vk::CommandPoolCreateFlags::empty();
        if command_pool_def.transient {
            command_pool_create_flags |= ash::vk::CommandPoolCreateFlags::TRANSIENT;
        }

        let pool_create_info = ash::vk::CommandPoolCreateInfo::builder()
            .flags(command_pool_create_flags)
            .queue_family_index(queue_family_index);

        let vk_command_pool = unsafe {
            device_context
                .vk_device()
                .create_command_pool(&pool_create_info, None)?
        };

        Ok(Self { vk_command_pool })
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .destroy_command_pool(self.vk_command_pool, None);
        }
    }
}

impl CommandPool {
    pub(crate) fn reset_command_pool_platform(&self) -> GfxResult<()> {
        unsafe {
            self.inner.device_context.vk_device().reset_command_pool(
                self.inner.backend_command_pool.vk_command_pool,
                ash::vk::CommandPoolResetFlags::empty(),
            )?;
        }
        Ok(())
    }

    pub(crate) fn vk_command_pool(&self) -> ash::vk::CommandPool {
        self.inner.backend_command_pool.vk_command_pool
    }
}
