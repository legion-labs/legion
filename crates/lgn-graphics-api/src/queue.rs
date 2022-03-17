use lgn_tracing::span_fn;

use crate::{
    backends::BackendQueue, CommandBuffer, CommandPool, CommandPoolDef, DeviceContext, Fence,
    GfxResult, PagedBufferAllocation, PresentSuccessResult, Semaphore, Swapchain,
};

/// Used to indicate which type of queue to use. Some operations require certain
/// types of queues.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum QueueType {
    /// Graphics queues generally supports all operations and are a safe default
    /// choice
    Graphics,

    /// Compute queues can be used for compute-based work.
    Compute,

    /// Transfer queues are generally limited to basic operations like copying
    /// data from buffers to images.
    Transfer,
}

pub(crate) struct QueueInner {
    device_context: DeviceContext,
    queue_type: QueueType,
    pub(crate) backend_queue: BackendQueue,
}

pub struct Queue {
    pub(crate) inner: Box<QueueInner>,
}

impl Queue {
    pub fn new(device_context: &DeviceContext, queue_type: QueueType) -> GfxResult<Self> {
        let backend_queue = BackendQueue::new(device_context, queue_type)?;

        Ok(Self {
            inner: Box::new(QueueInner {
                device_context: device_context.clone(),
                queue_type,
                backend_queue,
            }),
        })
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn queue_type(&self) -> QueueType {
        self.inner.queue_type
    }

    pub fn family_index(&self) -> u32 {
        self.backend_family_index()
    }

    pub fn create_command_pool(&self, command_pool_def: &CommandPoolDef) -> GfxResult<CommandPool> {
        CommandPool::new(self, command_pool_def)
    }

    #[span_fn]
    pub fn submit(
        &self,
        command_buffers: &[&CommandBuffer],
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
    ) -> GfxResult<()> {
        self.backend_submit(
            command_buffers,
            wait_semaphores,
            signal_semaphores,
            signal_fence,
        )
    }

    #[span_fn]
    pub fn present(
        &self,
        swapchain: &Swapchain,
        wait_semaphores: &[&Semaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult> {
        self.backend_present(
            &self.inner.device_context,
            swapchain,
            wait_semaphores,
            image_index,
        )
    }

    pub fn wait_for_queue_idle(&self) -> GfxResult<()> {
        self.backend_wait_for_queue_idle()
    }

    pub fn commit_sparse_bindings<'a>(
        &self,
        prev_frame_semaphore: &'a Semaphore,
        unbind_pages: &[PagedBufferAllocation],
        unbind_semaphore: &'a Semaphore,
        bind_pages: &[PagedBufferAllocation],
        bind_semaphore: &'a Semaphore,
    ) -> &'a Semaphore {
        self.backend_commit_sparse_bindings(
            prev_frame_semaphore,
            unbind_pages,
            unbind_semaphore,
            bind_pages,
            bind_semaphore,
        )
    }
}
