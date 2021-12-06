use lgn_graphics_api::{CommandBuffer, CommandBufferDef, CommandPool, CommandPoolDef, Queue};

use super::OnFrameEventHandler;
use crate::RenderHandle;

// TODO: CommandBuffer should be boxed.
pub type CommandBufferHandle = RenderHandle<CommandBuffer>;

pub(crate) struct CommandBufferPool {
    command_pool: CommandPool,
    availables: Vec<CommandBuffer>,
    in_flights: Vec<CommandBuffer>,
    acquired_count: u32,
}

impl CommandBufferPool {
    pub(crate) fn new(queue: &Queue) -> Self {
        Self {
            command_pool: queue
                .create_command_pool(&CommandPoolDef { transient: true })
                .unwrap(),
            availables: Vec::new(),
            in_flights: Vec::new(),
            acquired_count: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.command_pool.reset_command_pool().unwrap();
        self.availables.append(&mut self.in_flights);
    }

    pub(crate) fn acquire(&mut self) -> CommandBufferHandle {
        let result = if self.availables.is_empty() {
            let def = CommandBufferDef {
                is_secondary: false,
            };
            self.command_pool.create_command_buffer(&def).unwrap()
        } else {
            self.availables.pop().unwrap()
        };
        self.acquired_count += 1;
        CommandBufferHandle::new(result)
    }

    pub(crate) fn release(&mut self, mut handle: CommandBufferHandle) {
        assert!(handle.is_valid());
        assert!(self.acquired_count > 0);
        self.in_flights.push(handle.take());
        self.acquired_count -= 1;
    }
}

impl OnFrameEventHandler for CommandBufferPool {
    fn on_begin_frame(&mut self) {
        self.reset();
    }

    fn on_end_frame(&mut self) {
        assert_eq!(self.acquired_count, 0);
    }
}

pub(crate) type CommandBufferPoolHandle = RenderHandle<CommandBufferPool>;
