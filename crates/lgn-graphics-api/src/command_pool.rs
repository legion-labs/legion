use crate::{
    backends::BackendCommandPool, CommandBuffer, CommandBufferDef, DeviceContext, GfxResult, Queue,
    QueueType,
};

/// Used to create a `CommandPool`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandPoolDef {
    /// Set to true if the command buffers allocated from the pool are expected
    /// to have very short lifetimes
    pub transient: bool,
}

pub struct CommandPool {
    pub(crate) device_context: DeviceContext,
    pub(crate) queue_type: QueueType,
    pub(crate) queue_family_index: u32,
    pub(crate) backend_command_pool: BackendCommandPool,
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        self.backend_command_pool.destroy(&self.device_context);
    }
}

impl CommandPool {
    pub fn new(queue: &Queue, command_pool_def: CommandPoolDef) -> GfxResult<Self> {
        let device_context = queue.device_context().clone();
        let backend_command_pool =
            BackendCommandPool::new(&device_context, queue, command_pool_def)?;

        Ok(Self {
            device_context,
            queue_type: queue.queue_type(),
            queue_family_index: queue.family_index(),
            backend_command_pool,
        })
    }

    pub fn create_command_buffer(&mut self, command_buffer_def: CommandBufferDef) -> CommandBuffer {
        CommandBuffer::new(self.device_context(), self, command_buffer_def)
    }

    pub fn reset_command_pool(&mut self) -> GfxResult<()> {
        self.reset_command_pool_platform()
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.device_context
    }

    pub fn queue_type(&self) -> QueueType {
        self.queue_type
    }

    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }
}
