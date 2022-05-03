use std::{
    alloc::Layout,
    ptr::NonNull,
    sync::{Arc, Mutex},
};

use lgn_core::Handle;
use lgn_graphics_api::{
    Buffer, BufferCreateFlags, BufferDef, BufferViewDef, DeviceContext, IndexBufferBinding,
    IndexType, MemoryUsage, ResourceUsage, TransientBufferView, VertexBufferBinding,
};

use super::GpuSafePool;

pub struct TransientBufferAllocation {
    buffer: Buffer,
    byte_offset: u64,
    size: u64,
}

#[allow(unsafe_code)]
unsafe impl Send for TransientBufferAllocation {}

impl TransientBufferAllocation {
    #[allow(unsafe_code)]
    pub fn buffer(&self) -> &Buffer {
        unsafe { self.buffer.as_ref() }
    }

    pub fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    #[allow(unsafe_code)]
    pub fn ptr(&self) -> *mut u8 {
        unsafe {
            self.buffer
                .as_ref()
                .mapped_ptr()
                .add(self.byte_offset as usize)
        }
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding {
        VertexBufferBinding::new(self.buffer(), self.byte_offset)
    }

    pub fn index_buffer_binding(&self, index_type: IndexType) -> IndexBufferBinding {
        IndexBufferBinding::new(self.buffer(), self.byte_offset, index_type)
    }

    pub fn to_buffer_view(&self, view_def: BufferViewDef) -> TransientBufferView {
        self.buffer().create_transient_view(view_def)
    }
}

const TRANSIENT_BUFFER_BLOCK_SIZE: u64 = 64 * 1024;

struct TransientBuffer {
    device_context: DeviceContext,
    buffer: Buffer,
    byte_offset: u64,
    capacity: u64,
}

impl TransientBuffer {
    fn new(device_context: &DeviceContext, size: u64) -> Self {
        let required_size =
            lgn_utils::memory::round_size_up_to_alignment_u64(size, TRANSIENT_BUFFER_BLOCK_SIZE);

        let buffer = device_context.create_buffer(BufferDef {
				name: "PageHeap".to_string(),
            size: required_size,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_UNORDERED_ACCESS
                | ResourceUsage::AS_CONST_BUFFER
                | ResourceUsage::AS_VERTEX_BUFFER
                | ResourceUsage::AS_INDEX_BUFFER,
            create_flags: BufferCreateFlags::empty(),
            memory_usage: MemoryUsage::CpuToGpu,
            always_mapped: true,
        });

        Self {
            device_context: device_context.clone(),
            buffer,
            byte_offset: 0,
            capacity: required_size,
        }
    }

    pub fn begin_frame(&mut self) {
        self.byte_offset = 0;
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn byte_offset(&self) -> u64 {
        self.byte_offset
            }

    pub fn size(&self) -> u64 {
        self.capacity - self.byte_offset
    }

    fn allocate(
        &mut self,
        data_layout: Layout,
        resource_usage: ResourceUsage,
    ) -> Option<TransientBufferAllocation> {
        let min_alignment = if resource_usage == ResourceUsage::AS_CONST_BUFFER {
            self.device_context
                .device_info()
                .min_uniform_buffer_offset_alignment
        } else {
            self.device_context
                .device_info()
                .min_storage_buffer_offset_alignment
        };
        let required_alignment = u64::from(min_alignment).max(data_layout.align() as u64);
        let required_size = data_layout.size() as u64;
        let aligned_offset =
            lgn_utils::memory::round_size_up_to_alignment_u64(self.byte_offset, required_alignment);
        let next_offset = aligned_offset + required_size;

        if next_offset > self.capacity {
            None
        } else {
            self.byte_offset = next_offset;

            Some(TransientBufferAllocation {
                buffer: NonNull::from(&self.buffer),
                byte_offset: aligned_offset,
                size: required_size,
            })
            }
        }
}

// impl OnFrameEventHandler for TransientBuffer {
//     fn on_begin_frame(&mut self) {
//         self.byte_offset = 0;
//     }

//     fn on_end_frame(&mut self) {}
// }

pub(crate) struct TransientPagedBufferInner {
    device_context: DeviceContext,
    transient_buffers: GpuSafePool<TransientBuffer>,
    frame_pool: Vec<Handle<TransientBuffer>>,
}

#[derive(Clone)]
pub struct TransientPagedBuffer {
    inner: Arc<Mutex<TransientPagedBufferInner>>,
}

impl TransientPagedBuffer {
    pub fn new(device_context: &DeviceContext, num_cpu_frames: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TransientPagedBufferInner {
                device_context: device_context.clone(),
                transient_buffers: GpuSafePool::new(num_cpu_frames),
                frame_pool: Vec::new(),
            })),
        }
    }

    pub fn begin_frame(&mut self) {
        let mut inner = self.inner.lock().unwrap();

        assert!(inner.frame_pool.is_empty());

        inner
            .transient_buffers
            .begin_frame(TransientBuffer::begin_frame);
                }

    pub fn end_frame(&mut self) {
        let mut inner = self.inner.lock().unwrap();

        let mut frame_pool = std::mem::take(&mut inner.frame_pool);

        frame_pool.drain(..).for_each(|mut handle| {
            inner.transient_buffers.release(handle.transfer());
        });

        inner.transient_buffers.end_frame(|_| ());
            }

    fn acquire_page(&self, min_page_size: u64) -> Handle<TransientBuffer> {
        let inner = &mut *self.inner.lock().unwrap();

        for (i, handle) in inner.frame_pool.iter().enumerate() {
            if min_page_size <= handle.size() {
                return inner.frame_pool.swap_remove(i);
        }
    }

        inner
            .transient_buffers
            .acquire_or_create(|| TransientBuffer::new(&inner.device_context, min_page_size))
    }

    fn release_page(&self, page: Handle<TransientBuffer>) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.frame_pool.push(page);
        }
}

pub struct TransientBufferAllocator {
    device_context: DeviceContext,
    paged_buffer: TransientPagedBuffer,
    allocation: Handle<TransientBuffer>,
}

impl TransientBufferAllocator {
    pub fn new(
        device_context: &DeviceContext,
        paged_buffer: &TransientPagedBuffer,
        min_alloc_size: u64,
    ) -> Self {
        let allocation = paged_buffer.acquire_page(min_alloc_size);
        Self {
            paged_buffer: paged_buffer.clone(),
            allocation,
            device_context: device_context.clone(),
        }
    }

    pub fn copy_data<T>(
        &mut self,
        data: &T,
        resource_usage: ResourceUsage,
    ) -> TransientBufferAllocation {
        self.copy_data_slice(std::slice::from_ref(data), resource_usage)
        }

    pub fn copy_data_slice<T>(
        &mut self,
        data: &[T],
        resource_usage: ResourceUsage,
    ) -> TransientBufferAllocation {
        let data_layout = Layout::array::<T>(data.len()).unwrap();
        let src = data.as_ptr().cast::<u8>();
        let dst = self.allocate(data_layout, resource_usage);

        #[allow(unsafe_code)]
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst.ptr(), data_layout.size());
        }

        dst
    }

    fn allocate(
        &mut self,
        data_layout: Layout,
        resource_usage: ResourceUsage,
    ) -> TransientBufferAllocation {
        let mut allocation = self.allocation.allocate(data_layout, resource_usage);

        while allocation.is_none() {
            self.paged_buffer.release_page(self.allocation.transfer());
            self.allocation = self.paged_buffer.acquire_page(data_layout.size() as u64);
            allocation = self.allocation.allocate(data_layout, resource_usage);
        }

        allocation.unwrap()
        }
}

impl Drop for TransientBufferAllocator {
    fn drop(&mut self) {
        self.paged_buffer.release_page(self.allocation.transfer());
    }
}
