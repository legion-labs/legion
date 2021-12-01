use std::sync::{Arc, Mutex};

use graphics_api::{
    Buffer, BufferDef, BufferView, BufferViewDef, CommandBuffer, DeviceContext, DeviceInfo,
    IndexBufferBinding, IndexType, MemoryAllocation, MemoryAllocationDef, MemoryUsage, QueueType,
    ResourceCreation, ResourceUsage, VertexBufferBinding,
};

use super::{Range, RangeAllocator};

pub struct TransientSubAllocation {
    buffer: Buffer,
    data: *mut u8,
    offset: u64,
    size: u64,
}

impl TransientSubAllocation {
    pub fn bind_allocation_as_vertex_buffer(&self, cmd_buffer: &CommandBuffer) {
        cmd_buffer
            .cmd_bind_vertex_buffers(
                0,
                &[VertexBufferBinding {
                    buffer: &self.buffer,
                    byte_offset: self.offset,
                }],
            )
            .unwrap();
    }

    pub fn bind_allocation_as_index_buffer(
        &self,
        cmd_buffer: &CommandBuffer,
        index_type: IndexType,
    ) {
        cmd_buffer
            .cmd_bind_index_buffer(&IndexBufferBinding {
                buffer: &self.buffer,
                byte_offset: self.offset,
                index_type,
            })
            .unwrap();
    }

    pub fn const_buffer_view_for_allocation(&self) -> BufferView {
        let buffer_view_def = BufferViewDef::as_const_buffer_with_offset(self.size, self.offset);

        BufferView::from_buffer(&self.buffer, &buffer_view_def).unwrap()
    }
}

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
            queue_type: QueueType::Graphics,
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
            range_allocator: RangeAllocator::new(num_pages),
            page_size,
            frame_allocations: PageAllocationsForFrame {
                frame: 0,
                allocations: Vec::<Range>::new(),
            },
            all_allocations: Vec::<PageAllocationsForFrame>::new(),
        }
    }

    pub fn allocate_page(&mut self, size: u64) -> Option<TransientSubAllocation> {
        let alloc_size = legion_utils::memory::round_size_up_to_alignment_u64(size, self.page_size);
        let num_pages = alloc_size / self.page_size;

        match self.range_allocator.allocate(num_pages) {
            None => None,
            Some(range) => {
                self.frame_allocations.allocations.push(range);
                Some(TransientSubAllocation {
                    buffer: self.buffer.clone(),
                    data: self.buffer_memory.mapped_ptr(),
                    offset: range.first * self.page_size,
                    size: alloc_size,
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

    pub fn allocate_page(&self, size: u64) -> TransientSubAllocation {
        let mut inner = self.inner.lock().unwrap();

        for page_heap in &mut inner.page_heaps {
            if let Some(allocation) = page_heap.allocate_page(size) {
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
    allocation: TransientSubAllocation,
    device_info: DeviceInfo,
    offset: u64,
}

impl TransientBufferAllocator {
    pub fn new(
        device_context: &DeviceContext,
        paged_buffer: &TransientPagedBuffer,
        min_alloc_size: u64,
    ) -> Self {
        Self {
            paged_buffer: paged_buffer.clone(),
            allocation: paged_buffer.allocate_page(min_alloc_size),
            device_info: *device_context.device_info(),
            offset: 0,
        }
    }

    pub fn allocate(
        &mut self,
        data_size_in_bytes: u64,
        resource_usage: ResourceUsage,
    ) -> TransientSubAllocation {
        let alignment = if resource_usage == ResourceUsage::AS_CONST_BUFFER {
            self.device_info.min_uniform_buffer_offset_alignment
        } else {
            self.device_info.min_storage_buffer_offset_alignment
        };

        let old_offset =
            legion_utils::memory::round_size_up_to_alignment_u64(self.offset, u64::from(alignment));
        self.offset += legion_utils::memory::round_size_up_to_alignment_u64(
            data_size_in_bytes,
            u64::from(alignment),
        );

        if self.offset > self.allocation.size {
            self.allocation = self.paged_buffer.allocate_page(data_size_in_bytes);
            self.offset = 0;
        }

        TransientSubAllocation {
            buffer: self.allocation.buffer.clone(),
            data: self.allocation.data,
            offset: self.allocation.offset + old_offset,
            size: self.offset - old_offset,
        }
    }

    pub fn copy_data<T: Copy>(
        &mut self,
        data: &[T],
        resource_usage: ResourceUsage,
    ) -> TransientSubAllocation {
        let data_size_in_bytes = legion_utils::memory::slice_size_in_bytes(data) as u64;

        let allocation = self.allocate(data_size_in_bytes, resource_usage);
        let src = data.as_ptr().cast::<u8>();

        #[allow(unsafe_code)]
        unsafe {
            std::ptr::copy_nonoverlapping(
                src,
                allocation.data.add(allocation.offset as usize),
                data_size_in_bytes as usize,
            );
        }

        allocation
    }
}
