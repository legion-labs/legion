use std::{
    alloc::Layout,
    cell::{Cell, RefCell},
    sync::{Arc, Mutex},
};

use lgn_graphics_api::{
    Buffer, BufferAllocation, BufferDef, DeviceContext, DeviceInfo, MemoryAllocation,
    MemoryAllocationDef, MemoryUsage, Range, ResourceCreation, ResourceUsage,
};

use super::RangeAllocator;

#[derive(Clone)]
struct PageAllocationsForFrame {
    frame: u64,
    allocations: Vec<Range>,
}

struct PageHeap {
    buffer: Buffer,
    buffer_memory: MemoryAllocation,
    range_allocator: RangeAllocator,
    page_size: u64,
    frame_allocations: PageAllocationsForFrame,
    all_allocations: Vec<PageAllocationsForFrame>,
}

impl PageHeap {
    pub fn new(device_context: &DeviceContext, num_pages: u64, page_size: u64) -> Self {
        let buffer_def = BufferDef {
            size: page_size * num_pages,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_CONST_BUFFER
                | ResourceUsage::AS_VERTEX_BUFFER
                | ResourceUsage::AS_INDEX_BUFFER,
            creation_flags: ResourceCreation::empty(),
        };

        let buffer = device_context.create_buffer(&buffer_def);

        let alloc_def = MemoryAllocationDef {
            memory_usage: MemoryUsage::CpuToGpu,
            always_mapped: true,
        };

        let buffer_memory = MemoryAllocation::from_buffer(device_context, &buffer, &alloc_def);

        Self {
            buffer,
            buffer_memory,
            range_allocator: RangeAllocator::new(page_size * num_pages),
            page_size,
            frame_allocations: PageAllocationsForFrame {
                frame: 0,
                allocations: Vec::<Range>::new(),
            },
            all_allocations: Vec::<PageAllocationsForFrame>::new(),
        }
    }

    pub fn allocate_page(&mut self, layout: Layout) -> Option<BufferAllocation> {
        assert!(layout.align() as u64 <= self.page_size);

        let alloc_size =
            lgn_utils::memory::round_size_up_to_alignment_u64(layout.size() as u64, self.page_size);

        match self.range_allocator.allocate(alloc_size) {
            None => None,
            Some(range) => {
                self.frame_allocations.allocations.push(range);
                Some(BufferAllocation {
                    buffer: self.buffer.clone(),
                    memory: self.buffer_memory.clone(),
                    range,
                })
            }
        }
    }

    pub fn begin_frame(&mut self, current_cpu_frame: u64, last_complete_gpu_frame: u64) {
        if !self.all_allocations.is_empty() {
            let mut remove_indexes = Vec::new();
            let mut index_offset = 0;

            for index in 0..self.all_allocations.len() {
                let allocations = &self.all_allocations[index];
                if allocations.frame <= last_complete_gpu_frame {
                    for allocation in &allocations.allocations {
                        self.range_allocator.free(*allocation);
                    }
                    remove_indexes.push(index - index_offset);
                    index_offset += 1;
                }
            }

            for index in remove_indexes {
                self.all_allocations.remove(index);
            }
        }

        self.all_allocations.push(self.frame_allocations.clone());

        self.frame_allocations = PageAllocationsForFrame {
            frame: current_cpu_frame,
            allocations: Vec::<Range>::new(),
        }
    }
}

pub(crate) struct TransientPagedBufferInner {
    page_heaps: Vec<PageHeap>,
    current_cpu_frame: u64,
    last_complete_gpu_frame: u64,
}

#[derive(Clone)]
pub struct TransientPagedBuffer {
    inner: Arc<Mutex<TransientPagedBufferInner>>,
}

impl TransientPagedBuffer {
    pub fn new(device_context: &DeviceContext, num_pages: u64, page_size: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TransientPagedBufferInner {
                page_heaps: vec![PageHeap::new(device_context, num_pages, page_size)],
                current_cpu_frame: 3,
                last_complete_gpu_frame: 0,
            })),
        }
    }

    pub fn allocate_page(&self, layout: Layout) -> BufferAllocation {
        let mut inner = self.inner.lock().unwrap();

        for page_heap in &mut inner.page_heaps {
            if let Some(allocation) = page_heap.allocate_page(layout) {
                return allocation;
            }
        }

        panic!();
    }

    pub fn begin_frame(&self) {
        let mut inner = self.inner.lock().unwrap();

        inner.current_cpu_frame += 1;
        inner.last_complete_gpu_frame += 1;

        let current_cpu_frame = inner.current_cpu_frame;
        let last_complete_gpu_frame = inner.last_complete_gpu_frame;

        for page_heap in &mut inner.page_heaps {
            page_heap.begin_frame(current_cpu_frame, last_complete_gpu_frame);
        }
    }
}

pub struct TransientBufferAllocator {
    paged_buffer: TransientPagedBuffer,
    allocation: RefCell<BufferAllocation>,
    device_info: DeviceInfo,
    offset: Cell<u64>,
}

impl TransientBufferAllocator {
    pub fn new(
        device_context: &DeviceContext,
        paged_buffer: &TransientPagedBuffer,
        min_alloc_size: u64,
    ) -> Self {
        let allocation = paged_buffer
            .allocate_page(Layout::from_size_align(min_alloc_size as usize, 64 * 1024).unwrap());
        let offset = allocation.offset();
        Self {
            paged_buffer: paged_buffer.clone(),
            allocation: RefCell::new(allocation),
            device_info: *device_context.device_info(),
            offset: Cell::new(offset),
        }
    }

    pub fn allocate(&self, data_layout: Layout, resource_usage: ResourceUsage) -> BufferAllocation {
        let alignment = if resource_usage == ResourceUsage::AS_CONST_BUFFER {
            self.device_info.min_uniform_buffer_offset_alignment
        } else {
            self.device_info.min_storage_buffer_offset_alignment
        };
        let alignment = u64::from(alignment).max(data_layout.align() as u64);

        let mut aligned_offset =
            lgn_utils::memory::round_size_up_to_alignment_u64(self.offset.get(), alignment);
        let aligned_size =
            lgn_utils::memory::round_size_up_to_alignment_u64(data_layout.size() as u64, alignment);
        let mut new_offset = aligned_offset + aligned_size;
        let mut allocation = self.allocation.borrow_mut();

        if new_offset > allocation.size() {
            *allocation = self.paged_buffer.allocate_page(data_layout);

            aligned_offset = allocation.offset();
            new_offset = aligned_offset + aligned_size;

            assert!(
                aligned_offset
                    == lgn_utils::memory::round_size_up_to_alignment_u64(aligned_offset, alignment)
            );
        }

        self.offset.set(new_offset);

        BufferAllocation {
            buffer: allocation.buffer.clone(),
            memory: allocation.memory.clone(),
            range: Range {
                first: aligned_offset,
                last: new_offset,
            },
        }
    }

    pub fn copy_data<T: Copy>(&self, data: &T, resource_usage: ResourceUsage) -> BufferAllocation {
        let data_layout = std::alloc::Layout::new::<T>();
        let allocation = self.allocate(data_layout, resource_usage);
        let src = (data as *const T).cast::<u8>();

        #[allow(unsafe_code)]
        unsafe {
            std::ptr::copy_nonoverlapping(
                src,
                allocation
                    .memory
                    .mapped_ptr()
                    .add(allocation.offset() as usize),
                data_layout.size(),
            );
        }
        allocation
    }

    pub fn copy_data_slice<T: Copy>(
        &self,
        data: &[T],
        resource_usage: ResourceUsage,
    ) -> BufferAllocation {
        // let layout = lgn_utils::memory::slice_size_in_bytes(data) as u64;
        let data_layout = Layout::array::<T>(data.len()).unwrap();
        let allocation = self.allocate(data_layout, resource_usage);
        let src = data.as_ptr().cast::<u8>();

        #[allow(unsafe_code)]
        unsafe {
            std::ptr::copy_nonoverlapping(
                src,
                allocation
                    .memory
                    .mapped_ptr()
                    .add(allocation.offset() as usize),
                data_layout.size(),
            );
        }
        allocation
    }
}
