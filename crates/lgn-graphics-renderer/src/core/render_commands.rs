use std::{alloc::Layout, slice, sync::Arc};

use crossbeam_channel::{Receiver, Sender};
use egui::mutex::RwLock;
use lgn_core::{Handle, ObjectPool};
use lgn_graphics_api::Buffer;

type BoxedRenderCommand = Box<dyn RenderCommand + 'static>;

struct Inner {
    render_commands_pool: RwLock<ObjectPool<RenderCommandQueue>>,
    sender: Sender<BoxedRenderCommand>,
    receiver: Receiver<BoxedRenderCommand>,
}

#[derive(Clone)]
pub struct RenderCommandManager {
    inner: Arc<Inner>,
}

impl RenderCommandManager {
    pub fn new() -> Self {
        let (sx, rx) = crossbeam_channel::unbounded();
        Self {
            inner: Arc::new(Inner {
                render_commands_pool: RwLock::new(ObjectPool::new()),
                sender: sx,
                receiver: rx,
            }),
        }
    }

    pub fn acquire(&self) -> Handle<RenderCommandQueue> {
        let mut render_commands_pool = self.inner.render_commands_pool.write();
        render_commands_pool.acquire_or_create(|| RenderCommandQueue {
            sender: self.inner.sender.clone(),
        })
    }

    pub fn release(&self, handle: Handle<RenderCommandQueue>) {
        let mut render_commands_pool = self.inner.render_commands_pool.write();
        render_commands_pool.release(handle);
    }

    pub fn flush(&self) {
        let receiver = &self.inner.receiver;

        for mut render_command in receiver.try_iter() {
            render_command.execute();
        }
    }
}

#[allow(unsafe_code)]
unsafe impl Send for RenderCommandManager {}

#[allow(unsafe_code)]
unsafe impl Sync for RenderCommandManager {}

pub trait RenderCommand: Send + 'static {
    fn execute(&mut self);
}

pub struct UpdateGPUBuffer {
    pub src_buffer: Vec<u8>,
    pub dst_buffer: Buffer,
    pub dst_offset: u32,
}

impl RenderCommand for UpdateGPUBuffer {
    fn execute(&mut self) {}
}

pub struct RenderCommandQueue {
    
    sender: Sender<BoxedRenderCommand>,
}

impl RenderCommandQueue {
    pub fn send<T: RenderCommand + 'static>(&mut self, command: T) {
        self.sender.send(Box::new(command)).unwrap();
    }
}

pub struct BinaryWriter {
    buf: Vec<u8>,
}

impl BinaryWriter {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn write<T: Sized>(&mut self, data: &T) -> usize {
        self.write_slice(std::slice::from_ref(data))
    }

    #[allow(unsafe_code)]
    pub fn write_slice<T: Sized>(&mut self, data: &[T]) -> usize {
        let layout = Layout::array::<T>(data.len()).unwrap();
        let data_ptr = data.as_ptr().cast::<u8>();
        let data_slice = unsafe { slice::from_raw_parts(data_ptr, layout.size()) };
        self.buf.extend_from_slice(data_slice);
        data_slice.len()
    }

    pub fn take(mut self) -> Vec<u8> {
        self.buf
    }
}
