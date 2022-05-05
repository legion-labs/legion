use lgn_graphics_api::Buffer;

pub struct BufferUpdate {
    pub src_buffer: Vec<u8>,
    pub dst_buffer: Buffer,
    pub dst_offset: u64,
}

pub struct GpuUploadManager {}

impl GpuUploadManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn push(&mut self, update: BufferUpdate) {
        dbg!("x");
    }
}
