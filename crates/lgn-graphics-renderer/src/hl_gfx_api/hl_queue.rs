use lgn_graphics_api::{
    CommandBuffer, Fence, GfxResult, PresentSuccessResult, Queue, Semaphore, Swapchain,
};
use lgn_tracing::span_fn;

pub struct HLQueue<'rc> {
    queue: &'rc Queue,
}

impl<'rc> HLQueue<'rc> {
    pub(crate) fn new(queue: &'rc Queue) -> Self {
        Self { queue }
    }

    #[span_fn]
    pub fn submit(
        &self,
        command_buffers: &[&CommandBuffer],
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
    ) {
        self.queue
            .submit(
                command_buffers,
                wait_semaphores,
                signal_semaphores,
                signal_fence,
            )
            .unwrap();
    }

    pub fn present(
        &self,
        swapchain: &Swapchain,
        wait_semaphores: &[&Semaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult> {
        self.queue.present(swapchain, wait_semaphores, image_index)
    }

    #[span_fn]
    pub fn wait_for_queue_idle(&self) -> GfxResult<()> {
        self.queue.wait_for_queue_idle()
    }
}
