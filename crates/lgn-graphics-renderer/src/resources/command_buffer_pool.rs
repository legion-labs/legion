use std::sync::Arc;

use lgn_core::Handle;
use lgn_graphics_api::{CommandBuffer, CommandBufferDef, CommandPool, CommandPoolDef, Queue};
use parking_lot::Mutex;

use crate::GraphicsQueue;

use super::GpuSafePool;

pub type CommandBufferHandle = Handle<CommandBuffer>;

pub struct CommandBufferPool {
    command_pool: CommandPool,
    availables: Vec<CommandBuffer>,
    in_flights: Vec<CommandBuffer>,
    acquired_count: u32,
}

impl CommandBufferPool {
    pub(crate) fn new(queue: &Queue) -> Self {
        Self {
            command_pool: queue.create_command_pool(CommandPoolDef { transient: true }),
            availables: Vec::new(),
            in_flights: Vec::new(),
            acquired_count: 0,
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.command_pool.reset_command_pool().unwrap();
        let availables = &mut self.availables;
        let in_flights = &mut self.in_flights;

        availables.append(in_flights);
    }

    pub(crate) fn end_frame(&mut self) {
        assert_eq!(self.acquired_count, 0);
    }

    pub(crate) fn acquire(&mut self) -> CommandBufferHandle {
        let result = if self.availables.is_empty() {
            self.command_pool.create_command_buffer(CommandBufferDef {
                is_secondary: false,
            })
        } else {
            self.availables.pop().unwrap()
        };
        self.acquired_count += 1;

        CommandBufferHandle::new(result)
    }

    pub(crate) fn release(&mut self, mut handle: CommandBufferHandle) {
        assert!(self.acquired_count > 0);
        self.in_flights.push(handle.take());
        self.acquired_count -= 1;
    }
}

struct Inner {
    command_buffer_pools: Mutex<GpuSafePool<CommandBufferPool>>,
    graphics_queue: GraphicsQueue,
}

#[derive(Clone)]
pub struct TransientCommandBufferManager {
    inner: Arc<Inner>,
}

impl TransientCommandBufferManager {
    pub fn new(num_render_frames: u64, graphics_queue: &GraphicsQueue) -> Self {
        Self {
            inner: Arc::new(Inner {
                command_buffer_pools: Mutex::new(GpuSafePool::new(num_render_frames)),
                graphics_queue: graphics_queue.clone(),
            }),
        }
    }

    pub fn begin_frame(&self, frame_index: usize) {
        let mut command_buffer_pools = self.inner.command_buffer_pools.lock();
        command_buffer_pools.begin_frame(frame_index, CommandBufferPool::begin_frame);
    }

    pub fn end_frame(&self, frame_index: usize) {
        let mut command_buffer_pools = self.inner.command_buffer_pools.lock();
        command_buffer_pools.end_frame(frame_index, CommandBufferPool::end_frame);
    }

    pub fn acquire(&self) -> Handle<CommandBufferPool> {
        let mut command_buffer_pools = self.inner.command_buffer_pools.lock();
        command_buffer_pools
            .acquire_or_create(|| CommandBufferPool::new(&self.inner.graphics_queue.queue()))
    }

    pub fn release(&self, handle: Handle<CommandBufferPool>) {
        let mut command_buffer_pools = self.inner.command_buffer_pools.lock();
        command_buffer_pools.release(handle);
    }
}

pub struct TransientCommandBufferAllocator {
    command_buffer_manager: TransientCommandBufferManager,
    command_buffer_pool: Handle<CommandBufferPool>,
}

impl TransientCommandBufferAllocator {
    pub fn new(manager: &TransientCommandBufferManager) -> Self {
        Self {
            command_buffer_manager: manager.clone(),
            command_buffer_pool: manager.acquire(),
        }
    }

    pub fn acquire(&mut self) -> CommandBufferHandle {
        self.command_buffer_pool.acquire()
    }

    pub fn release(&mut self, handle: CommandBufferHandle) {
        self.command_buffer_pool.release(handle);
    }
}

impl Drop for TransientCommandBufferAllocator {
    fn drop(&mut self) {
        self.command_buffer_manager
            .release(self.command_buffer_pool.transfer());
    }
}
