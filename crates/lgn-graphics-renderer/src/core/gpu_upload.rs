use lgn_graphics_api::prelude::*;

use crate::RenderContext;

pub struct BufferUpdate {
    pub src_buffer: Vec<u8>,
    pub dst_buffer: Buffer,
    pub dst_offset: u64,
}

pub struct GpuUploadManager {
    updates: Vec<BufferUpdate>,
}

impl GpuUploadManager {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    pub fn push(&mut self, update: BufferUpdate) {
        self.updates.push(update);
    }

    pub fn upload(&mut self, render_context: &mut RenderContext<'_>) {
        if self.updates.is_empty() {
            return;
        }

        let mut cmd_buffer_handle = render_context.transient_commandbuffer_allocator.acquire();
        let cmd_buffer = cmd_buffer_handle.as_mut();

        cmd_buffer.begin();

        for update in self.updates.drain(..) {
            let transient_alloc = render_context
                .transient_buffer_allocator
                .copy_data_slice(&update.src_buffer, ResourceUsage::empty());

            cmd_buffer.cmd_resource_barrier(
                &[BufferBarrier {
                    buffer: &update.dst_buffer,
                    src_state: ResourceState::SHADER_RESOURCE,
                    dst_state: ResourceState::COPY_DST,
                    queue_transition: BarrierQueueTransition::None,
                }],
                &[],
            );

            cmd_buffer.cmd_copy_buffer_to_buffer(
                transient_alloc.buffer(),
                &update.dst_buffer,
                &[BufferCopy {
                    src_offset: transient_alloc.byte_offset(),
                    dst_offset: update.dst_offset,
                    size: update.src_buffer.len() as u64,
                }],
            );

            cmd_buffer.cmd_resource_barrier(
                &[BufferBarrier {
                    buffer: &update.dst_buffer,
                    src_state: ResourceState::COPY_DST,
                    dst_state: ResourceState::SHADER_RESOURCE,
                    queue_transition: BarrierQueueTransition::None,
                }],
                &[],
            );
        }

        cmd_buffer.end();

        render_context
            .graphics_queue
            .queue_mut()
            .submit(&[cmd_buffer], &[], &[], None);

        render_context
            .transient_commandbuffer_allocator
            .release(cmd_buffer_handle);
    }
}
