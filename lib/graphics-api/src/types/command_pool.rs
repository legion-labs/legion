#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanCommandPool;
use crate::{
    CommandBuffer, CommandBufferDef, CommandPoolDef, DeviceContext, GfxResult, Queue, QueueType,
};

pub(crate) struct CommandPoolInner {
    pub(crate) device_context: DeviceContext,
    pub(crate) queue_type: QueueType,
    pub(crate) queue_family_index: u32,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_command_pool: VulkanCommandPool,
}

pub struct CommandPool {
    pub(crate) inner: Box<CommandPoolInner>,
}

impl Drop for CommandPoolInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_command_pool.destroy(&self.device_context);
    }
}

impl CommandPool {
    pub fn new(queue: &Queue, command_pool_def: &CommandPoolDef) -> GfxResult<Self> {
        let device_context = queue.device_context().clone();
        #[cfg(feature = "vulkan")]
        let platform_command_pool =
            VulkanCommandPool::new(&device_context, queue.platform_queue(), command_pool_def)
                .map_err(|e| {
                    log::error!("Error creating command pool {:?}", e);
                    ash::vk::Result::ERROR_UNKNOWN
                })?;

        Ok(Self {
            inner: Box::new(CommandPoolInner {
                device_context,
                queue_type: queue.queue_type(),
                queue_family_index: queue.queue_family_index(),
                #[cfg(any(feature = "vulkan"))]
                platform_command_pool,
            }),
        })
    }

    pub fn create_command_buffer(
        &self,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<CommandBuffer> {
        CommandBuffer::new(self.device_context(), self, command_buffer_def)
    }

    pub fn reset_command_pool(&self) -> GfxResult<()> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.reset_command_pool_platform()
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn queue_type(&self) -> QueueType {
        self.inner.queue_type
    }

    pub fn queue_family_index(&self) -> u32 {
        self.inner.queue_family_index
    }
}
