use lgn_graphics_api::{
    CommandBuffer, Fence, GfxResult, PagedBufferAllocation, PresentSuccessResult, Queue, Semaphore,
    Swapchain,
};
use lgn_tracing::trace_function;
use parking_lot::RwLockReadGuard;

use crate::resources::{CommandBufferHandle, CommandBufferPool};

pub struct HLQueue<'rc> {
    queue: RwLockReadGuard<'rc, Queue>,
    command_buffer_pool: &'rc CommandBufferPool,
}

impl<'rc> HLQueue<'rc> {
    pub(crate) fn new(
        queue: RwLockReadGuard<'rc, Queue>,
        command_buffer_pool: &'rc CommandBufferPool,
    ) -> Self {
        Self {
            queue,
            command_buffer_pool,
        }
    }

    #[trace_function]
    pub fn submit(
        &self,
        command_buffer_handles: &mut [CommandBufferHandle],
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
    ) {
        {
            let mut command_buffers = smallvec::SmallVec::<[&CommandBuffer; 16]>::with_capacity(
                command_buffer_handles.len(),
            );

            for cbh in command_buffer_handles.iter() {
                command_buffers.push(cbh);
            }

            self.queue
                .submit(
                    &command_buffers,
                    wait_semaphores,
                    signal_semaphores,
                    signal_fence,
                )
                .unwrap();
        }

        for cbh in command_buffer_handles.iter_mut() {
            self.command_buffer_pool.release(cbh.transfer());
        }
    }

    pub fn present(
        &self,
        swapchain: &Swapchain,
        wait_semaphores: &[&Semaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult> {
        self.queue.present(swapchain, wait_semaphores, image_index)
    }

    #[trace_function]
    pub fn wait_for_queue_idle(&self) -> GfxResult<()> {
        self.queue.wait_for_queue_idle()
    }

    pub fn commmit_sparse_bindings<'a>(
        &self,
        prev_frame_semaphore: &'a Semaphore,
        unbind_pages: &[PagedBufferAllocation],
        unbind_semaphore: &'a Semaphore,
        bind_pages: &[PagedBufferAllocation],
        bind_semaphore: &'a Semaphore,
    ) -> &'a Semaphore {
        self.queue.commmit_sparse_bindings(
            prev_frame_semaphore,
            unbind_pages,
            unbind_semaphore,
            bind_pages,
            bind_semaphore,
        )
    }
}
