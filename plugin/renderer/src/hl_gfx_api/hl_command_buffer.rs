use lgn_graphics_api::CommandBuffer;

pub struct HLCommandBuffer {
    cmd_buffer: CommandBuffer,
}

impl HLCommandBuffer {
    fn new(cmd_buffer: CommandBuffer) -> Self {
        Self { cmd_buffer }
    }
}
