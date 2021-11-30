use graphics_api::{CommandBuffer, CommandPool, Queue, CommandPoolDef, CommandBufferDef};

use crate::RendererHandle;

use super::Rotate;

pub type CommandBufferHandle = RendererHandle<CommandBuffer>;

pub(crate) struct CommandBufferPool {
    command_pool: CommandPool,
    availables: Vec<CommandBuffer>,
    in_flights: Vec<CommandBuffer>,
}

impl CommandBufferPool {
    pub(crate) fn new(queue: &Queue) -> Self {
        Self {
            command_pool: queue
                .create_command_pool(&CommandPoolDef { transient: true })
                .unwrap(),
            availables: Vec::new(),
            in_flights: Vec::new(),
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
        CommandBufferHandle::new(result)
    }

    pub(crate) fn release(&mut self, mut handle: CommandBufferHandle) {
        assert!(handle.is_valid());
        self.in_flights.push(handle.peek());
    }
}

impl Rotate for CommandBufferPool {
    fn rotate(&mut self) {
        self.reset();
    }
}

pub(crate) type CommandBufferPoolHandle = RendererHandle<CommandBufferPool>;