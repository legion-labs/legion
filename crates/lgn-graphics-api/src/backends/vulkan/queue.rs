use ash::vk;
use lgn_tracing::trace;

use super::internal::VkQueue;
use crate::{
    CommandBuffer, DeviceContext, Fence, GfxResult, PresentSuccessResult, Queue, QueueType,
    Semaphore, SemaphoreUsage, Swapchain,
};

pub(crate) struct VulkanQueue {
    pub(crate) queue: VkQueue,
}

impl VulkanQueue {
    pub fn new(device_context: &DeviceContext, queue_type: QueueType) -> Self {
        let queue = match queue_type {
            QueueType::Graphics => device_context
                .queue_allocator()
                .allocate_graphics_queue(device_context),
            QueueType::Compute => device_context
                .queue_allocator()
                .allocate_compute_queue(device_context),
            QueueType::Transfer => device_context
                .queue_allocator()
                .allocate_transfer_queue(device_context),
        }
        .ok_or_else(|| format!("All queues of type {:?} already allocated", queue_type))
        .unwrap();

        Self { queue }
    }
}

impl Queue {
    #[inline]
    pub(crate) fn vk_queue(&self) -> &VkQueue {
        &self.backend_queue.queue
    }

    pub(crate) fn backend_family_index(&self) -> u32 {
        self.vk_queue().queue_family_index()
    }

    // Make sure we always use the dedicated queue if it exists
    fn present_to_given_or_dedicated_queue(
        &self,
        device_context: &DeviceContext,
        swapchain: &Swapchain,
        present_info: &vk::PresentInfoKHR,
    ) -> GfxResult<bool> {
        let is_suboptimal =
            if let Some(dedicated_present_queue) = swapchain.dedicated_present_queue() {
                // Because of the way we search for present-compatible queues, we don't
                // necessarily have the same underlying mutex in all instances of a
                // dedicated present queue. So fallback to a single global lock
                let _dedicated_present_lock = device_context
                    .dedicated_present_queue_lock()
                    .lock()
                    .unwrap();
                unsafe {
                    trace!(
                        "present to dedicated present queue {:?}",
                        dedicated_present_queue
                    );
                    swapchain
                        .vk_swapchain_loader()
                        .queue_present(dedicated_present_queue, present_info)?
                }
            } else {
                let queue = self.vk_queue().queue().lock().unwrap();
                trace!("present to dedicated present queue {:?}", *queue);
                unsafe {
                    swapchain
                        .vk_swapchain_loader()
                        .queue_present(*queue, present_info)?
                }
            };

        Ok(is_suboptimal)
    }

    pub fn backend_submit(
        &self,
        command_buffers: &[&CommandBuffer],
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
        current_cpu_frame: u64,
    ) {
        let mut command_buffer_list = Vec::with_capacity(command_buffers.len());
        for command_buffer in command_buffers.iter() {
            command_buffer_list.push(command_buffer.vk_command_buffer());
        }

        let mut wait_semaphore_list = Vec::with_capacity(wait_semaphores.len());
        let mut timeline_submit_list = Vec::with_capacity(wait_semaphores.len());
        let mut wait_dst_stage_mask = Vec::with_capacity(wait_semaphores.len());

        for wait_semaphore in wait_semaphores {
            let timeline = wait_semaphore
                .definition()
                .usage_flags
                .intersects(SemaphoreUsage::TIMELINE);

            // Don't wait on a semaphore that will never signal
            //TODO: Assert or fail here?
            if timeline || wait_semaphore.signal_available() {
                timeline_submit_list.push(current_cpu_frame);

                wait_semaphore_list.push(wait_semaphore.vk_semaphore());
                wait_dst_stage_mask.push(vk::PipelineStageFlags::ALL_COMMANDS);

                wait_semaphore.set_signal_available(false);
            }
        }

        let mut signal_semaphore_list = Vec::with_capacity(signal_semaphores.len());
        let mut signal_submit_list = Vec::with_capacity(signal_semaphores.len());

        for signal_semaphore in signal_semaphores {
            let timeline = signal_semaphore
                .definition()
                .usage_flags
                .intersects(SemaphoreUsage::TIMELINE);

            // Don't signal a semaphore if something is already going to signal it
            //TODO: Assert or fail here?
            if timeline || !signal_semaphore.signal_available() {
                signal_submit_list.push(current_cpu_frame);

                signal_semaphore_list.push(signal_semaphore.vk_semaphore());
                signal_semaphore.set_signal_available(true);
            }
        }

        let mut timeline_submit_info = vk::TimelineSemaphoreSubmitInfo::builder()
            .wait_semaphore_values(&timeline_submit_list)
            .signal_semaphore_values(&signal_submit_list);

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphore_list)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .signal_semaphores(&signal_semaphore_list)
            .command_buffers(&command_buffer_list)
            .push_next(&mut timeline_submit_info);

        let fence = signal_fence.map_or(vk::Fence::null(), Fence::vk_fence);
        unsafe {
            let queue = self.vk_queue().queue().lock().unwrap();
            trace!(
                "submit {} command buffers to queue {:?}",
                command_buffer_list.len(),
                *queue
            );
            self.vk_queue()
                .device_context()
                .vk_device()
                .queue_submit(*queue, &[*submit_info], fence)
                .unwrap();
        }

        if let Some(signal_fence) = signal_fence {
            signal_fence.set_submitted(true);
        }
    }

    pub fn backend_present(
        &self,
        device_context: &DeviceContext,
        swapchain: &Swapchain,
        wait_semaphores: &[&Semaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult> {
        let mut wait_semaphore_list = Vec::with_capacity(wait_semaphores.len());
        for wait_semaphore in wait_semaphores {
            if wait_semaphore.signal_available() {
                wait_semaphore_list.push(wait_semaphore.vk_semaphore());
                wait_semaphore.set_signal_available(false);
            }
        }

        let swapchains = [swapchain.vk_swapchain()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphore_list)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        //TODO: PresentInfoKHRBuilder::results() is only useful for presenting multiple
        // swapchains - presumably that's for multiwindow cases.

        let result =
            self.present_to_given_or_dedicated_queue(device_context, swapchain, &*present_info);

        match result {
            Ok(is_suboptimial) => {
                if is_suboptimial {
                    Ok(PresentSuccessResult::SuccessSuboptimal)
                } else {
                    Ok(PresentSuccessResult::Success)
                }
            }
            #[allow(clippy::match_single_binding)]
            Err(e) => match e {
                // todo(jal)
                //GfxError::VkError(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                //    Ok(PresentSuccessResult::DeviceReset)
                //}
                e => Err(e),
            },
        }
    }

    pub fn backend_wait_for_queue_idle(&self) {
        let queue = self.vk_queue().queue().lock().unwrap();
        unsafe {
            self.vk_queue()
                .device_context()
                .vk_device()
                .queue_wait_idle(*queue)
                .unwrap();
        }
    }
}
