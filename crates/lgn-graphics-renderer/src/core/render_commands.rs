use std::{alloc::Layout, slice, sync::Arc};

use egui::mutex::RwLock;
use lgn_core::{Handle, ObjectPool};

use super::RenderResources;

struct Inner {
    render_commands_pool: RwLock<ObjectPool<RenderCommandQueue>>,
}

#[derive(Clone)]
pub struct RenderCommandManager {
    inner: Arc<Inner>,
}

impl RenderCommandManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                render_commands_pool: RwLock::new(ObjectPool::new()),
            }),
        }
    }

    pub fn builder(&self) -> RenderCommandBuilder {
        RenderCommandBuilder::new(self)
    }

    pub fn apply(&self, render_resources: &RenderResources) {
        let mut render_commands_pool = self.inner.render_commands_pool.write();
        for queue in render_commands_pool.iter_mut() {
            queue.apply(render_resources);
        }
    }

    fn acquire(&self) -> Handle<RenderCommandQueue> {
        let mut render_commands_pool = self.inner.render_commands_pool.write();
        render_commands_pool.acquire_or_create(RenderCommandQueue::new)
    }

    fn release(&self, handle: Handle<RenderCommandQueue>) {
        let mut render_commands_pool = self.inner.render_commands_pool.write();
        render_commands_pool.release(handle);
    }
}

pub struct RenderCommandBuilder {
    render_command_manager: RenderCommandManager,
    render_command_queue: Handle<RenderCommandQueue>,
}

impl RenderCommandBuilder {
    pub fn new(render_command_manager: &RenderCommandManager) -> Self {
        Self {
            render_command_manager: render_command_manager.clone(),
            render_command_queue: render_command_manager.acquire(),
        }
    }

    pub fn push<C: RenderCommand>(&mut self, command: C) {
        self.render_command_queue.push(command);
    }
}

impl Drop for RenderCommandBuilder {
    fn drop(&mut self) {
        self.render_command_manager
            .release(self.render_command_queue.transfer());
    }
}

pub trait RenderCommand: Send + 'static {
    fn execute(self, render_resources: &RenderResources);
}

struct RenderCommandMeta {
    offset: usize,
    func: unsafe fn(value: *mut u8, world: &RenderResources),
}

pub struct RenderCommandQueue {
    metas: Vec<RenderCommandMeta>,
    bytes: Vec<u8>,
}

impl RenderCommandQueue {
    pub fn new() -> Self {
        Self {
            metas: Vec::new(),
            bytes: Vec::new(),
        }
    }

    #[allow(unsafe_code)]
    pub fn push<C: RenderCommand>(&mut self, command: C) {
        unsafe fn execute_command<T: RenderCommand>(
            command: *mut u8,
            render_resources: &RenderResources,
        ) {
            let command = command.cast::<T>().read_unaligned();
            command.execute(render_resources);
        }

        let size = std::mem::size_of::<C>();
        let old_len = self.bytes.len();

        self.metas.push(RenderCommandMeta {
            offset: old_len,
            func: execute_command::<C>,
        });

        if size > 0 {
            self.bytes.reserve(size);

            unsafe {
                std::ptr::copy_nonoverlapping(
                    std::ptr::addr_of!(command).cast::<u8>(),
                    self.bytes.as_mut_ptr().add(old_len),
                    size,
                );
                self.bytes.set_len(old_len + size);
            }
        }

        std::mem::forget(command);
    }

    #[allow(unsafe_code)]
    pub fn apply(&mut self, render_resources: &RenderResources) {
        // SAFE: In the iteration below, `meta.func` will safely consume and drop each
        // pushed command. This operation is so that we can reuse the bytes
        // `Vec<u8>`'s internal storage and prevent unnecessary allocations.
        unsafe {
            self.bytes.set_len(0);
        };

        let byte_ptr = if self.bytes.as_mut_ptr().is_null() {
            // SAFE: If the vector's buffer pointer is `null` this mean nothing has been
            // pushed to its bytes. This means either that:
            //
            // 1) There are no commands so this pointer will never be read/written from/to.
            //
            // 2) There are only zero-sized commands pushed.
            //    According to https://doc.rust-lang.org/std/ptr/index.html
            //    "The canonical way to obtain a pointer that is valid for zero-sized
            // accesses is NonNull::dangling"    therefore it is safe to call
            // `read_unaligned` on a pointer produced from `NonNull::dangling` for
            //    zero-sized commands.
            unsafe { std::ptr::NonNull::dangling().as_mut() }
        } else {
            self.bytes.as_mut_ptr()
        };

        for meta in self.metas.drain(..) {
            // SAFE: The implementation of `write_command` is safe for the according Command
            // type. The bytes are safely cast to their original type, safely
            // read, and then dropped.
            unsafe {
                (meta.func)(byte_ptr.add(meta.offset), render_resources);
            }
        }
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

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
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

    pub fn take(self) -> Vec<u8> {
        self.buf
    }
}
