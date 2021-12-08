#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanQueue;
use crate::{
    CommandBuffer, CommandPool, CommandPoolDef, DeviceContext, Fence, GfxResult,
    PagedBufferAllocation, PresentSuccessResult, QueueType, Semaphore, Swapchain,
};

struct QueueInner {
    device_context: DeviceContext,
    queue_type: QueueType,

    #[cfg(feature = "vulkan")]
    platform_queue: VulkanQueue,
}

pub struct Queue {
    inner: Box<QueueInner>,
}

impl Queue {
    pub fn new(device_context: &DeviceContext, queue_type: QueueType) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_queue = VulkanQueue::new(device_context, queue_type).map_err(|e| {
            log::error!("Error creating buffer {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        Ok(Self {
            inner: Box::new(QueueInner {
                device_context: device_context.clone(),
                queue_type,
                #[cfg(any(feature = "vulkan"))]
                platform_queue,
            }),
        })
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn queue_type(&self) -> QueueType {
        self.inner.queue_type
    }

    pub fn queue_family_index(&self) -> u32 {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_queue().queue_family_index()
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_queue(&self) -> &VulkanQueue {
        &self.inner.platform_queue
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
        self.inner.platform_queue.submit(
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
        self.inner.platform_queue.present(
            &self.inner.device_context,
            swapchain,
            wait_semaphores,
            image_index,
        )
    }

    pub fn wait_for_queue_idle(&self) -> GfxResult<()> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.inner.platform_queue.wait_for_queue_idle()
    }

    pub fn commmit_sparse_bindings<'a>(
        &self,
        prev_frame_semaphore: &'a Semaphore,
        unbind_pages: &[PagedBufferAllocation],
        unbind_semaphore: &'a Semaphore,
        bind_pages: &[PagedBufferAllocation],
        bind_semaphore: &'a Semaphore,
    ) -> &'a Semaphore {
        #[cfg(any(feature = "vulkan"))]
        self.inner.platform_queue.commmit_sparse_bindings(
            prev_frame_semaphore,
            unbind_pages,
            unbind_semaphore,
            bind_pages,
            bind_semaphore,
        )
    }
}
