use ash::vk;

use super::internal::VkQueue;
use super::{SparseBindingInfo, VulkanDeviceContext, VulkanSwapchain};
use crate::{
    CommandBuffer, DeviceContext, Fence, GfxResult, PagedBufferAllocation, PresentSuccessResult,
    QueueType, Semaphore, Swapchain,
};

pub(crate) struct VulkanQueue {
    queue: VkQueue,
}

impl VulkanQueue {
    pub fn vk_queue(&self) -> &VkQueue {
        &self.queue
    }

    pub fn new(device_context: &DeviceContext, queue_type: QueueType) -> GfxResult<Self> {
        let queue = match queue_type {
            QueueType::Graphics => device_context
                .platform_device_context()
                .queue_allocator()
                .allocate_graphics_queue(device_context),
            QueueType::Compute => device_context
                .platform_device_context()
                .queue_allocator()
                .allocate_compute_queue(device_context),
            QueueType::Transfer => device_context
                .platform_device_context()
                .queue_allocator()
                .allocate_transfer_queue(device_context),
            QueueType::Decode => device_context
                .platform_device_context()
                .queue_allocator()
                .allocate_decode_queue(device_context),
            QueueType::Encode => device_context
                .platform_device_context()
                .queue_allocator()
                .allocate_encode_queue(device_context),
        }
        .ok_or_else(|| format!("All queues of type {:?} already allocated", queue_type))?;

        Ok(Self { queue })
    }

    // Make sure we always use the dedicated queue if it exists
    pub(self) fn present_to_given_or_dedicated_queue(
        &self,
        vulkan_device_context: &VulkanDeviceContext,
        swapchain: &VulkanSwapchain,
        present_info: &vk::PresentInfoKHR,
    ) -> GfxResult<bool> {
        let is_suboptimal =
            if let Some(dedicated_present_queue) = swapchain.dedicated_present_queue() {
                // Because of the way we search for present-compatible queues, we don't necessarily have
                // the same underlying mutex in all instances of a dedicated present queue. So fallback
                // to a single global lock
                let _dedicated_present_lock = vulkan_device_context
                    .dedicated_present_queue_lock()
                    .lock()
                    .unwrap();
                unsafe {
                    log::trace!(
                        "present to dedicated present queue {:?}",
                        dedicated_present_queue
                    );
                    swapchain
                        .vk_swapchain_loader()
                        .queue_present(dedicated_present_queue, present_info)?
                }
            } else {
                let queue = self.queue.queue().lock().unwrap();
                log::trace!("present to dedicated present queue {:?}", *queue);
                unsafe {
                    swapchain
                        .vk_swapchain_loader()
                        .queue_present(*queue, present_info)?
                }
            };

        Ok(is_suboptimal)
    }

    pub fn queue_family_index(&self) -> u32 {
        self.queue.queue_family_index()
    }

    pub fn submit(
        &self,
        command_buffers: &[&CommandBuffer],
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
    ) -> GfxResult<()> {
        let mut command_buffer_list = Vec::with_capacity(command_buffers.len());
        for command_buffer in command_buffers {
            command_buffer_list.push(command_buffer.platform_command_buffer().vk_command_buffer());
        }

        let mut wait_semaphore_list = Vec::with_capacity(wait_semaphores.len());
        let mut wait_dst_stage_mask = Vec::with_capacity(wait_semaphores.len());
        for wait_semaphore in wait_semaphores {
            wait_semaphore_list.push(wait_semaphore.platform_semaphore().vk_semaphore());
            wait_dst_stage_mask.push(vk::PipelineStageFlags::ALL_COMMANDS);
        }

        let mut signal_semaphore_list = Vec::with_capacity(signal_semaphores.len());
        for signal_semaphore in signal_semaphores {
            signal_semaphore_list.push(signal_semaphore.platform_semaphore().vk_semaphore());
        }

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphore_list)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .signal_semaphores(&signal_semaphore_list)
            .command_buffers(&command_buffer_list);

        let fence = signal_fence.map_or(vk::Fence::null(), Fence::vk_fence);
        unsafe {
            let queue = self.queue.queue().lock().unwrap();
            log::trace!(
                "submit {} command buffers to queue {:?}",
                command_buffer_list.len(),
                *queue
            );
            self.queue
                .device_context()
                .device()
                .queue_submit(*queue, &[*submit_info], fence)?;
        }

        if let Some(signal_fence) = signal_fence {
            signal_fence.set_submitted(true);
        }

        Ok(())
    }

    pub fn present(
        &self,
        device_context: &DeviceContext,
        swapchain: &Swapchain,
        wait_semaphores: &[&Semaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult> {
        let mut wait_semaphore_list = Vec::with_capacity(wait_semaphores.len());
        for wait_semaphore in wait_semaphores {
            wait_semaphore_list.push(wait_semaphore.platform_semaphore().vk_semaphore());
        }

        let swapchains = [swapchain.platform_swap_chain().vk_swapchain()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphore_list)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        //TODO: PresentInfoKHRBuilder::results() is only useful for presenting multiple swapchains -
        // presumably that's for multiwindow cases.

        let result = self.present_to_given_or_dedicated_queue(
            device_context.platform_device_context(),
            swapchain.platform_swap_chain(),
            &*present_info,
        );

        match result {
            Ok(is_suboptimial) => {
                if is_suboptimial {
                    Ok(PresentSuccessResult::SuccessSuboptimal)
                } else {
                    Ok(PresentSuccessResult::Success)
                }
            }
            Err(e) => match e {
                // todo(jal)
                //GfxError::VkError(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                //    Ok(PresentSuccessResult::DeviceReset)
                //}
                e => Err(e),
            },
        }
    }

    pub fn wait_for_queue_idle(&self) -> GfxResult<()> {
        let queue = self.queue.queue().lock().unwrap();
        unsafe {
            self.queue
                .device_context()
                .device()
                .queue_wait_idle(*queue)?;
        }

        Ok(())
    }

    pub fn commmit_sparse_bindings<'a>(
        &self,
        prev_frame_semaphore: &'a Semaphore,
        unbind_pages: &[PagedBufferAllocation],
        unbind_semaphore: &'a Semaphore,
        bind_pages: &[PagedBufferAllocation],
        bind_semaphore: &'a Semaphore,
    ) -> &'a Semaphore {
        let queue = self.queue.queue().lock().unwrap();

        let vk_prev_frame_semaphores = [prev_frame_semaphore.platform_semaphore().vk_semaphore()];
        let vk_unbind_semaphores = [unbind_semaphore.platform_semaphore().vk_semaphore()];
        let vk_bind_semaphores = [bind_semaphore.platform_semaphore().vk_semaphore()];

        if !unbind_pages.is_empty() {
            let mut binding_infos = Vec::with_capacity(unbind_pages.len());
            let mut vk_unbindings = Vec::with_capacity(unbind_pages.len());

            for page in unbind_pages {
                let mut binding_info = SparseBindingInfo {
                    sparse_bindings: Vec::new(),
                    buffer_offset: page.offset(),
                    buffer: &page.buffer,
                    bind: false,
                };

                vk_unbindings.push(
                    page.memory
                        .platform_allocation()
                        .binding_info(&mut binding_info),
                );

                binding_infos.push(binding_info);
            }

            let unbind_info_builder = ash::vk::BindSparseInfo::builder()
                .buffer_binds(&vk_unbindings)
                .wait_semaphores(&vk_prev_frame_semaphores)
                .signal_semaphores(&vk_unbind_semaphores);

            unsafe {
                self.queue
                    .device_context()
                    .device()
                    .queue_bind_sparse(*queue, &[*unbind_info_builder], vk::Fence::null())
                    .unwrap();
            }
        }

        if !bind_pages.is_empty() {
            let mut binding_infos = Vec::with_capacity(bind_pages.len());
            let mut vk_bindings = Vec::with_capacity(bind_pages.len());

            for page in bind_pages {
                let mut binding_info = SparseBindingInfo {
                    sparse_bindings: Vec::new(),
                    buffer_offset: page.offset(),
                    buffer: &page.buffer,
                    bind: false,
                };

                vk_bindings.push(
                    page.memory
                        .platform_allocation()
                        .binding_info(&mut binding_info),
                );

                binding_infos.push(binding_info);
            }

            let mut bind_info_builder = ash::vk::BindSparseInfo::builder()
                .buffer_binds(&vk_bindings)
                .signal_semaphores(&vk_bind_semaphores);
            if unbind_pages.is_empty() {
                bind_info_builder = bind_info_builder.wait_semaphores(&vk_prev_frame_semaphores);
            } else {
                bind_info_builder = bind_info_builder.wait_semaphores(&vk_unbind_semaphores);
            }

            unsafe {
                self.queue
                    .device_context()
                    .device()
                    .queue_bind_sparse(*queue, &[*bind_info_builder], vk::Fence::null())
                    .unwrap();
            }
            bind_semaphore
        } else if !unbind_pages.is_empty() {
            unbind_semaphore
        } else {
            prev_frame_semaphore
        }
    }
}
