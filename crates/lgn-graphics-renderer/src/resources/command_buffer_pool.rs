use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

use lgn_core::Handle;
use lgn_graphics_api::{CommandBuffer, CommandBufferDef, CommandPool, CommandPoolDef, Queue};
use parking_lot::Mutex;

use crate::GraphicsQueue;

use super::GpuSafePool;

pub type CommandBufferHandle = Handle<CommandBuffer>;

pub struct CommandBufferPool {
    command_pool: CommandPool,
    availables: RefCell<Vec<CommandBuffer>>,
    in_flights: RefCell<Vec<CommandBuffer>>,
    acquired_count: Cell<u32>,
}

impl CommandBufferPool {
    pub(crate) fn new(queue: &Queue) -> Self {
        Self {
            command_pool: queue
                .create_command_pool(&CommandPoolDef { transient: true })
                .unwrap(),
            availables: RefCell::new(Vec::new()),
            in_flights: RefCell::new(Vec::new()),
            acquired_count: Cell::new(0),
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.command_pool.reset_command_pool().unwrap();
        let mut availables = self.availables.borrow_mut();
        let mut in_flights = self.in_flights.borrow_mut();

        availables.append(in_flights.as_mut());
    }

    pub(crate) fn end_frame(&mut self) {
        assert_eq!(self.acquired_count.get(), 0);
    }

    pub(crate) fn acquire(&self) -> CommandBufferHandle {
        let mut availables = self.availables.borrow_mut();

        let result = if availables.is_empty() {
            let def = CommandBufferDef {
                is_secondary: false,
            };
            self.command_pool.create_command_buffer(&def).unwrap()
        } else {
            availables.pop().unwrap()
        };
        let acquired_count = self.acquired_count.get();
        self.acquired_count.set(acquired_count + 1);

        CommandBufferHandle::new(result)
    }

    pub(crate) fn release(&self, mut handle: CommandBufferHandle) {
        assert!(self.acquired_count.get() > 0);
        let mut in_flights = self.in_flights.borrow_mut();
        in_flights.push(handle.take());
        let acquired_count = self.acquired_count.get();
        self.acquired_count.set(acquired_count - 1);
    }
}

pub type CommandBufferPoolHandle = Handle<CommandBufferPool>;

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

    pub fn begin_frame(&self) {
        let mut command_buffer_pools = self.inner.command_buffer_pools.lock();
        command_buffer_pools.begin_frame(CommandBufferPool::begin_frame);
    }

    pub fn end_frame(&self) {
        let mut command_buffer_pools = self.inner.command_buffer_pools.lock();
        command_buffer_pools.end_frame(CommandBufferPool::end_frame);
    }

    pub fn acquire(&self) -> CommandBufferPoolHandle {
        let mut command_buffer_pools = self.inner.command_buffer_pools.lock();
        command_buffer_pools
            .acquire_or_create(|| CommandBufferPool::new(self.inner.graphics_queue.queue()))
    }

    pub fn release(&self, handle: CommandBufferPoolHandle) {
        let mut command_buffer_pools = self.inner.command_buffer_pools.lock();
        command_buffer_pools.release(handle);
    }
}

pub struct TransientCommandBufferAllocator {
    command_buffer_manager: TransientCommandBufferManager,
    command_buffer_pool: CommandBufferPoolHandle,
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
