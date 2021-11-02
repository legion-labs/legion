use ash::vk;

use super::{VulkanApi, VulkanCommandBuffer, VulkanDeviceContext, VulkanQueue};
use crate::{CommandBufferDef, CommandPool, CommandPoolDef, GfxResult, Queue, QueueType};

pub struct VulkanCommandPool {
    device_context: VulkanDeviceContext,
    vk_command_pool: vk::CommandPool,
    queue_type: QueueType,
    queue_family_index: u32,
}

impl Drop for VulkanCommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device_context
                .device()
                .destroy_command_pool(self.vk_command_pool, None);
        }
    }
}

impl VulkanCommandPool {
    pub fn queue_type(&self) -> QueueType {
        self.queue_type
    }

    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }

    pub fn vk_command_pool(&self) -> vk::CommandPool {
        self.vk_command_pool
    }

    pub fn new(queue: &VulkanQueue, command_pool_def: &CommandPoolDef) -> GfxResult<Self> {
        let queue_family_index = queue.queue().queue_family_index();
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
            queue
                .device_context()
                .device()
                .create_command_pool(&pool_create_info, None)?
        };

        Ok(Self {
            device_context: queue.device_context().clone(),
            vk_command_pool,
            queue_type: queue.queue_type(),
            queue_family_index,
        })
    }
}

impl CommandPool<VulkanApi> for VulkanCommandPool {
    fn device_context(&self) -> &VulkanDeviceContext {
        &self.device_context
    }

    fn create_command_buffer(
        &self,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<VulkanCommandBuffer> {
        VulkanCommandBuffer::new(self, command_buffer_def)
    }

    fn reset_command_pool(&self) -> GfxResult<()> {
        unsafe {
            self.device_context
                .device()
                .reset_command_pool(self.vk_command_pool, vk::CommandPoolResetFlags::empty())?;
        }
        Ok(())
    }
}
