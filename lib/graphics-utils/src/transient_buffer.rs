use std::{
    ptr::slice_from_raw_parts,
    sync::{Arc, Mutex},
};

use legion_graphics_api::{
    Buffer, BufferDef, BufferView, BufferViewDef, DeviceContext, Fence, FenceStatus, MemoryUsage,
    Queue, QueueType, ResourceUsage,
};

const TRANSIENT_BUFFER_PAGE_SIZE: u64 = 1024 * 1024;

pub struct MappedTransientPages<'a> {
    pub data: &'a [u8],
    pub offset: u64,
}

pub(crate) struct TransientPagedBufferInner {
    ring_buffer: Buffer,
    ring_buffer_view: BufferView,
    current_offset: u64,
    fences: Vec<Fence>,
    fence_offsets: Vec<u64>,
}

#[derive(Clone)]
pub struct TransientPagedBuffer {
    inner: Arc<Mutex<TransientPagedBufferInner>>,
}

impl TransientPagedBuffer {
    pub fn new(device_context: &DeviceContext, num_pages: u64) -> Self {
        let buffer_def = BufferDef {
            size: TRANSIENT_BUFFER_PAGE_SIZE * num_pages,
            memory_usage: MemoryUsage::CpuToGpu,
            queue_type: QueueType::Graphics,
            always_mapped: true,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE,
        };

        let ring_buffer = device_context.create_buffer(&buffer_def).unwrap();

        let buffer_view_def = BufferViewDef::as_byte_address_buffer(&buffer_def, true);

        let ring_buffer_view = BufferView::from_buffer(&ring_buffer, &buffer_view_def).unwrap();

        Self {
            inner: Arc::new(Mutex::new(TransientPagedBufferInner {
                ring_buffer,
                ring_buffer_view,
                current_offset: 0,
                fences: Vec::new(),
                fence_offsets: Vec::new(),
            })),
        }
    }

    pub fn allocate_pages(&self, size: u64) -> MappedTransientPages<'_> {
        let mut buffer_inner = self.inner.lock().unwrap();

        let buffer_size = buffer_inner.ring_buffer.definition().size;
        let page_count = (size + (TRANSIENT_BUFFER_PAGE_SIZE - 1)) / TRANSIENT_BUFFER_PAGE_SIZE;
        let alloc_size = page_count * TRANSIENT_BUFFER_PAGE_SIZE;

        let mut old_offset = buffer_inner.current_offset;
        let mut next_offset = old_offset + alloc_size;

        if old_offset <= buffer_size && next_offset > buffer_size {
            // allocation wraps around end of ring buffer, need to reallocate to keep it contiguous
            old_offset = 0;
            next_offset = alloc_size;

            for offset in &mut buffer_inner.fence_offsets {
                *offset -= buffer_size;
            }
        }

        buffer_inner.current_offset = next_offset;

        if buffer_inner.fence_offsets.len() == 1 {
            assert!(next_offset < buffer_inner.fence_offsets[0]);
        } else if next_offset > buffer_inner.fence_offsets[0] {
            buffer_inner.fences[0].wait().unwrap();

            buffer_inner.fences.remove(0);
            buffer_inner.fence_offsets.remove(0);
        }

        let mapped_ptr = buffer_inner.ring_buffer.map_buffer().unwrap().data_ptr();

        #[allow(unsafe_code)]
        unsafe {
            MappedTransientPages {
                data: &*slice_from_raw_parts(
                    mapped_ptr.wrapping_offset((old_offset).try_into().unwrap()),
                    alloc_size as usize,
                ),
                offset: old_offset,
            }
        }
    }

    pub fn begin_frame(&self, device_context: &DeviceContext) {
        let mut buffer_inner = self.inner.lock().unwrap();

        // Add a new fence for this upcomming frame
        buffer_inner
            .fences
            .push(device_context.create_fence().unwrap());

        let offset = buffer_inner.current_offset;
        let buffer_size = buffer_inner.ring_buffer.definition().size;
        buffer_inner.fence_offsets.push(offset + buffer_size);

        // compact existing fences that are now open
        while buffer_inner.fences[0].get_fence_status().unwrap() == FenceStatus::Complete {
            buffer_inner.fences.remove(0);
            buffer_inner.fence_offsets.remove(0);
        }
    }

    pub fn end_frame(&self, graphics_queue: &Queue) {
        let buffer_inner = self.inner.lock().unwrap();

        graphics_queue
            .submit(&[], &[], &[], Some(buffer_inner.fences.last().unwrap()))
            .unwrap();
    }

    pub fn buffer_view(&self) -> BufferView {
        self.inner.lock().unwrap().ring_buffer_view.clone()
    }
}

pub struct TransientBufferAllocator<'a> {
    paged_buffer: TransientPagedBuffer,

    page_size: u64,
    current_page: MappedTransientPages<'a>,
    offset_in_page: u64,
}

impl<'a> TransientBufferAllocator<'a> {
    pub fn new(paged_buffer: &'a TransientPagedBuffer, initial_size: u64) -> Self {
        Self {
            paged_buffer: paged_buffer.clone(),
            page_size: initial_size,
            current_page: paged_buffer.allocate_pages(initial_size),
            offset_in_page: 0,
        }
    }

    pub fn copy_data<T: Copy>(&'a mut self, data: &[T]) -> u64 {
        let data_size_in_bytes = legion_utils::memory::slice_size_in_bytes(data) as u64;

        let mut old_offset = self.offset_in_page;
        let mut new_offset = self.offset_in_page + data_size_in_bytes;

        if new_offset > self.page_size {
            self.current_page = self.paged_buffer.allocate_pages(data_size_in_bytes);

            old_offset = 0;
            new_offset = old_offset + data_size_in_bytes;
        }
        self.offset_in_page = new_offset;

        let src = data.as_ptr().cast::<u8>();

        let required_alignment = std::mem::align_of::<T>();

        let dst: *mut u8 =
            std::ptr::addr_of!(self.current_page.data[old_offset as usize]) as *mut u8;
        assert_eq!(((dst as usize) % required_alignment), 0);

        #[allow(unsafe_code)]
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, data_size_in_bytes as usize);
        }

        self.current_page.offset + old_offset
    }
}
