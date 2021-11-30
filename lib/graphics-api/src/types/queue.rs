use crate::{
    CommandBuffer, CommandPool, CommandPoolDef, DeviceContext, Fence, GfxResult,
    PresentSuccessResult, QueueType, Semaphore, Swapchain,
};

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanQueue;

pub struct Queue {
    device_context: DeviceContext,
    queue_type: QueueType,

    #[cfg(feature = "vulkan")]
    platform_queue: VulkanQueue,
}

impl Queue {
    pub fn new(device_context: &DeviceContext, queue_type: QueueType) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_queue = VulkanQueue::new(device_context, queue_type).map_err(|e| {
            log::error!("Error creating buffer {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        Ok(Self {
            device_context: device_context.clone(),
            queue_type,
            #[cfg(any(feature = "vulkan"))]
            platform_queue,
        })
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.device_context
    }

    pub fn queue_type(&self) -> QueueType {
        self.queue_type
    }

    pub fn queue_family_index(&self) -> u32 {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_queue().queue_family_index()
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_queue(&self) -> &VulkanQueue {
        &self.platform_queue
    }

    pub fn create_command_pool(&self, command_pool_def: &CommandPoolDef) -> GfxResult<CommandPool> {
        CommandPool::new(self, command_pool_def)
    }

    pub fn submit(
        &self,
        command_buffers: &[&CommandBuffer],
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
    ) -> GfxResult<()> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_queue.submit(
            command_buffers,
            wait_semaphores,
            signal_semaphores,
            signal_fence,
        )
    }

    pub fn present(
        &self,
        swapchain: &Swapchain,
        wait_semaphores: &[&Semaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_queue.present(
            &self.device_context,
            swapchain,
            wait_semaphores,
            image_index,
        )
    }

    pub fn wait_for_queue_idle(&self) -> GfxResult<()> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_queue.wait_for_queue_idle()
    }
}
