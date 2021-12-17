use std::cell::{Cell, RefCell};

use lgn_graphics_api::{CommandBuffer, CommandBufferDef, CommandPool, CommandPoolDef, Queue};

use super::OnFrameEventHandler;
use crate::RenderHandle;

pub type CommandBufferHandle = RenderHandle<CommandBuffer>;

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

    pub(crate) fn reset(&mut self) {
        self.command_pool.reset_command_pool().unwrap();
        let mut availables = self.availables.borrow_mut();
        let mut in_flights = self.in_flights.borrow_mut();

        availables.append(in_flights.as_mut());
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

impl OnFrameEventHandler for CommandBufferPool {
    fn on_begin_frame(&mut self) {
        self.reset();
    }

    fn on_end_frame(&mut self) {
        assert_eq!(self.acquired_count.get(), 0);
    }
}

pub type CommandBufferPoolHandle = RenderHandle<CommandBufferPool>;
