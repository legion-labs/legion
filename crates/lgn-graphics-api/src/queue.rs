use lgn_tracing::span_fn;

use crate::{
    backends::BackendQueue, CommandBuffer, CommandPool, CommandPoolDef, DeviceContext, Fence,
    GfxResult, PresentSuccessResult, Semaphore, Swapchain,
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

pub struct Queue {
    device_context: DeviceContext,
    queue_type: QueueType,
    pub(crate) backend_queue: BackendQueue,
}

impl Queue {
    pub fn new(device_context: &DeviceContext, queue_type: QueueType) -> Self {
        let backend_queue = BackendQueue::new(device_context, queue_type);

        Self {
            device_context: device_context.clone(),
            queue_type,
            backend_queue,
        }
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.device_context
    }

    pub fn queue_type(&self) -> QueueType {
        self.queue_type
    }

    pub fn family_index(&self) -> u32 {
        self.backend_family_index()
    }

    pub fn create_command_pool(&self, command_pool_def: CommandPoolDef) -> CommandPool {
        CommandPool::new(self, command_pool_def)
    }

    #[span_fn]
    pub fn submit(
        &mut self,
        command_buffers: &[&CommandBuffer],
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
    ) {
        self.backend_submit(
            command_buffers,
            wait_semaphores,
            signal_semaphores,
            signal_fence,
        );
    }

    #[span_fn]
    pub fn present(
        &mut self,
        swapchain: &Swapchain,
        wait_semaphores: &[&Semaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult> {
        self.backend_present(
            &self.device_context,
            swapchain,
            wait_semaphores,
            image_index,
        )
    }

    pub fn wait_for_queue_idle(&mut self) {
        self.backend_wait_for_queue_idle();
    }
}
