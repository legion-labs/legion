use std::{
    alloc::Layout,
    cell::RefCell,
    marker::PhantomData,
    slice,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use egui::mutex::RwLock;
use lgn_core::Handle;
use lgn_utils::memory::round_size_up_to_alignment_usize;

use super::RenderResources;

struct CommandQueuePoolInner<CTX> {
    gamesim_pool: RwLock<Vec<Handle<RenderCommandQueue<CTX>>>>,
    acquired_count: AtomicUsize,
    exec_pool: RefCell<Vec<Handle<RenderCommandQueue<CTX>>>>,
}

impl<CTX> CommandQueuePoolInner<CTX> {
    fn new() -> Self {
        Self {
            gamesim_pool: RwLock::new(Vec::new()),
            acquired_count: AtomicUsize::new(0),
            exec_pool: RefCell::new(Vec::new()),
        }
    }
}

impl<CTX> Drop for CommandQueuePoolInner<CTX> {
    fn drop(&mut self) {
        assert_eq!(self.acquired_count.load(Ordering::SeqCst), 0);
        assert!(self.exec_pool.borrow().is_empty());
        let mut gamesim_pool = self.gamesim_pool.write();
        gamesim_pool.drain(..).for_each(|mut x| {
            x.take();
        });
    }
}

pub struct CommandQueuePool<CTX> {
    inner: Arc<CommandQueuePoolInner<CTX>>,
}

impl<CTX> CommandQueuePool<CTX> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CommandQueuePoolInner::new()),
        }
    }

    pub fn acquire(&self) -> Handle<RenderCommandQueue<CTX>> {
        let mut gamesim_pool = self.inner.gamesim_pool.write();
        let result = if gamesim_pool.is_empty() {
            Handle::new(RenderCommandQueue::new())
        } else {
            gamesim_pool.pop().unwrap()
        };
        self.inner.acquired_count.fetch_add(1, Ordering::SeqCst);
        result
    }

    pub fn release(&self, handle: Handle<RenderCommandQueue<CTX>>) {
        assert!(self.inner.acquired_count.load(Ordering::SeqCst) > 0);
        let mut gamesim_pool = self.inner.gamesim_pool.write();
        gamesim_pool.push(handle);
        self.inner.acquired_count.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn sync_update(&self) {
        assert_eq!(self.inner.acquired_count.load(Ordering::SeqCst), 0);

        let mut gamesim_pool = self.inner.gamesim_pool.write();

        let mut exec_pool_ref = self.inner.exec_pool.borrow_mut();
        let exec_pool = &mut *exec_pool_ref;

        let mut recycling_pool = std::mem::take(exec_pool);

        let mut i = 0;
        while i < gamesim_pool.len() {
            if !gamesim_pool[i].is_empty() {
                let queue = gamesim_pool.remove(i);
                exec_pool.push(queue);
            } else {
                i += 1;
            }
        }
        gamesim_pool.extend(recycling_pool.drain(..));
    }

    pub fn apply(&self, context: &CTX) {
        let mut exec_pool = self.inner.exec_pool.borrow_mut();
        for queue in exec_pool.iter_mut() {
            queue.apply(context);
        }
    }
}

impl<CTX> Clone for CommandQueuePool<CTX> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[allow(unsafe_code)]
unsafe impl<CTX> Send for CommandQueuePool<CTX> {}

#[allow(unsafe_code)]
unsafe impl<CTX> Sync for CommandQueuePool<CTX> {}

pub struct CommandBuilder<CTX> {
    pool: CommandQueuePool<CTX>,
    handle: Handle<RenderCommandQueue<CTX>>,
}

impl<CTX> CommandBuilder<CTX> {
    pub fn new(pool: &CommandQueuePool<CTX>) -> Self {
        Self {
            pool: pool.clone(),
            handle: pool.acquire(),
        }
    }

    pub fn push<C: RenderCommand<CTX>>(&mut self, command: C) {
        self.handle.push(command);
    }
}

impl<CTX> Drop for CommandBuilder<CTX> {
    fn drop(&mut self) {
        self.pool.release(self.handle.transfer());
    }
}

pub trait RenderCommand<CTX>: Send + 'static {
    fn execute(self, render_resources: &CTX);
}

struct RenderCommandMeta<CTX> {
    offset: usize,
    func: unsafe fn(value: *mut u8, world: &CTX),
}

#[derive(Default)]
pub struct RenderCommandQueue<CTX> {
    metas: Vec<RenderCommandMeta<CTX>>,
    bytes: Vec<u8>,
    _phantom: PhantomData<CTX>,
}

impl<CTX> RenderCommandQueue<CTX> {
    pub fn new() -> Self {
        Self {
            metas: Vec::new(),
            bytes: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.metas.is_empty()
    }

    #[allow(unsafe_code)]
    pub fn push<C: RenderCommand<CTX>>(&mut self, command: C) {
        unsafe fn execute_command<T: RenderCommand<CTX>, CTX>(command: *mut u8, context: &CTX) {
            let command = command.cast::<T>().read_unaligned();
            command.execute(context);
        }

        let size = std::mem::size_of::<C>();
        let old_len = self.bytes.len();

        self.metas.push(RenderCommandMeta {
            offset: old_len,
            func: execute_command::<C, CTX>,
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
    pub fn apply(&mut self, context: &CTX) {
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
                (meta.func)(byte_ptr.add(meta.offset), context);
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

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn align(&mut self, alignment: usize) -> usize {
        let new_len = round_size_up_to_alignment_usize(self.buf.len(), alignment);
        let padding = new_len - self.buf.len();
        if padding > 0 {
            self.buf.resize(new_len, 0xff);
        }
        padding
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

//
// RenderCommandManager
//

pub type RenderCommandQueuePool = CommandQueuePool<RenderResources>;
pub type RenderCommandBuilder = CommandBuilder<RenderResources>;

pub struct RenderCommandManager {
    pool: RenderCommandQueuePool,
}

impl RenderCommandManager {
    pub fn new(pool: &RenderCommandQueuePool) -> Self {
        Self { pool: pool.clone() }
    }

    pub fn sync_update(&mut self) {
        self.pool.sync_update();
    }

    pub fn apply(&mut self, render_resources: &RenderResources) {
        self.pool.apply(render_resources);
    }
}
