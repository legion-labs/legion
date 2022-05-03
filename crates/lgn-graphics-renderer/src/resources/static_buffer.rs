use std::sync::{Arc, Mutex, RwLock};

use lgn_graphics_api::{
    BarrierQueueTransition, Buffer, BufferBarrier, BufferCopy, BufferCreateFlags, BufferDef,
    BufferView, BufferViewDef, DeviceContext, IndexBufferBinding, IndexType, MemoryUsage,
    ResourceState, ResourceUsage, VertexBufferBinding,
};
use lgn_tracing::span_fn;

use super::{Range, RangeAllocator, TransientBufferAllocation, TransientBufferAllocator};
use crate::RenderContext;

const PAGE_SIZE: u64 = 256;

pub struct UnifiedStaticBuffer {
    buffer: Buffer,
    read_only_view: BufferView,
    allocator: UnifiedStaticBufferAllocator,
}

impl UnifiedStaticBuffer {
    pub fn new(device_context: &DeviceContext, virtual_buffer_size: u64) -> Self {
        let element_size = std::mem::size_of::<u32>() as u64;
        let element_count = virtual_buffer_size / element_size;
        let buffer_size = element_count * element_size;

        let buffer = device_context.create_buffer(BufferDef {
				name: "UnifiedStaticBuffer".to_string(),
            size: buffer_size,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_TRANSFERABLE
                | ResourceUsage::AS_VERTEX_BUFFER
                | ResourceUsage::AS_INDEX_BUFFER,
            create_flags: BufferCreateFlags::empty(),
                memory_usage: MemoryUsage::GpuOnly,
                always_mapped: false,
        });

        let read_only_view =
            buffer.create_view(BufferViewDef::as_byte_address_buffer(element_count, true));

        let allocator = UnifiedStaticBufferAllocator::new(&buffer, buffer_size);

        Self {
            buffer,
            read_only_view,
            allocator,
        }
    }

    pub fn allocator(&self) -> &UnifiedStaticBufferAllocator {
        &self.allocator
    }

    pub fn read_only_view(&self) -> &BufferView {
        &self.read_only_view
    }

    pub fn index_buffer_binding(&self) -> IndexBufferBinding {
        IndexBufferBinding::new(&self.buffer, 0, IndexType::Uint16)
    }
}

pub struct StaticBufferView {
    allocation: StaticBufferAllocation,
    buffer_view: BufferView,
}

impl StaticBufferView {
    fn new(allocation: &StaticBufferAllocation, view_definition: BufferViewDef) -> Self {
        let buffer_view = allocation.inner.buffer.create_view(view_definition);
        Self {
            allocation: allocation.clone(),
            buffer_view,
        }
        }

    pub fn buffer_view(&self) -> &BufferView {
        &self.buffer_view
    }
}

struct StaticBufferAllocationInner {
    buffer: Buffer,
    range: Range,
    allocator: UnifiedStaticBufferAllocator,
}
#[derive(Clone)]
pub(crate) struct StaticBufferAllocation {
    inner: Arc<StaticBufferAllocationInner>,
}

impl StaticBufferAllocation {
    fn new(allocator: &UnifiedStaticBufferAllocator, buffer: &Buffer, range: Range) -> Self {
        Self {
            inner: Arc::new(StaticBufferAllocationInner {
                buffer: buffer.clone(),
                range,
                allocator: allocator.clone(),
            }),
        }
    }

    pub fn byte_offset(&self) -> u64 {
        self.inner.range.begin()
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding {
        VertexBufferBinding::new(&self.inner.buffer, self.byte_offset())
    }

    pub fn create_view(&self, view_definition: BufferViewDef) -> StaticBufferView {
        let view_definition = BufferViewDef {
            byte_offset: self.inner.range.begin(),
            ..view_definition
        };
        StaticBufferView::new(self, view_definition)
    }
}

impl Drop for StaticBufferAllocationInner {
    fn drop(&mut self) {
        self.allocator.free(self.range);
    }
}

pub(crate) struct UnifiedStaticBufferAllocatorInner {
    buffer: Buffer,
    segment_allocator: RangeAllocator,
    job_blocks: Vec<GPUDataUpdaterCopy>,
}

#[derive(Clone)]
pub struct UnifiedStaticBufferAllocator {
    inner: Arc<Mutex<UnifiedStaticBufferAllocatorInner>>,
}

impl UnifiedStaticBufferAllocator {
    pub fn new(buffer: &Buffer, virtual_buffer_size: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(UnifiedStaticBufferAllocatorInner {
                buffer: buffer.clone(),
                segment_allocator: RangeAllocator::new(virtual_buffer_size),
                job_blocks: Vec::new(),
            })),
        }
    }

    pub(crate) fn allocate(&self, required_size: u64) -> StaticBufferAllocation {
        let inner = &mut *self.inner.lock().unwrap();

        let alloc_size =
            lgn_utils::memory::round_size_up_to_alignment_u64(required_size, PAGE_SIZE);

        if required_size != alloc_size {
            // TODO(vdbdd): use warn instead
            println!( "UnifiedStaticBufferAllocator: the segment required size ({} bytes) is less than the allocated size ({} bytes). {} bytes of memory will be wasted", segment_size, alloc_size, alloc_size-segment_size  );
        }

        let range = inner.segment_allocator.allocate(alloc_size).unwrap();

        StaticBufferAllocation::new(self, &inner.buffer, range)
        }

    fn free(&self, range: Range) {
        let inner = &mut *self.inner.lock().unwrap();
        inner.segment_allocator.free(range);
        }

    pub fn add_update_job_block(&self, mut job_blocks: Vec<GPUDataUpdaterCopy>) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.job_blocks.extend(job_blocks.drain(..));
    }

    #[span_fn]
    pub(crate) fn flush_updater(&self, render_context: &RenderContext<'_>) {
        let inner = &mut *self.inner.lock().unwrap();

        let graphics_queue = render_context.graphics_queue();

        let mut cmd_buffer = render_context.alloc_command_buffer();

        cmd_buffer.resource_barrier(
            &[BufferBarrier {
                buffer: &inner.buffer,
                src_state: ResourceState::SHADER_RESOURCE,
                dst_state: ResourceState::COPY_DST,
                queue_transition: BarrierQueueTransition::None,
            }],
            &[],
        );

        for job in &inner.job_blocks {
            cmd_buffer.copy_buffer_to_buffer(
                job.src_allocation.buffer(),
                &inner.buffer,
                &[BufferCopy {
                    src_offset: job.src_allocation.byte_offset(),
                    dst_offset: job.static_buffer_offset,
                    size: job.src_allocation.size(),
                }],
            );
        }
        inner.job_blocks.clear();

        cmd_buffer.resource_barrier(
            &[BufferBarrier {
                buffer: &inner.buffer,
                src_state: ResourceState::COPY_DST,
                dst_state: ResourceState::SHADER_RESOURCE,
                queue_transition: BarrierQueueTransition::None,
            }],
            &[],
        );

        graphics_queue.submit(&mut [cmd_buffer.finalize()], &[], &[], None);
    }
}

pub struct UniformGPUData<T> {
    allocated_pages: RwLock<Vec<StaticBufferAllocation>>,
    elements_per_page: u64,
    marker: std::marker::PhantomData<T>,
}

impl<T> UniformGPUData<T> {
    pub fn new(elements_per_page: u64) -> Self {
        Self {
            allocated_pages: RwLock::new(Vec::new()),
            elements_per_page,
            marker: ::std::marker::PhantomData,
        }
    }

    pub fn ensure_index_allocated(
        &self,
        allocator: &UnifiedStaticBufferAllocator,
        index: u32,
    ) -> u64 {
        let index_64 = u64::from(index);
        let element_size = std::mem::size_of::<T>() as u64;
        let elements_per_page = self.elements_per_page;
        let required_pages = (index_64 / elements_per_page) + 1;

        let index_of_page = index_64 / elements_per_page;
        let index_in_page = index_64 % elements_per_page;

        {
            let page_read_access = self.allocated_pages.read().unwrap();
            if page_read_access.len() >= required_pages as usize {
                return page_read_access[index_of_page as usize].byte_offset()
                    + (index_in_page * element_size);
            }
        }

        let mut page_write_access = self.allocated_pages.write().unwrap();

        while (page_write_access.len() as u64) < required_pages {
            let segment_size = elements_per_page * std::mem::size_of::<T>() as u64;
            page_write_access.push(allocator.allocate(segment_size));
        }

        page_write_access[index_of_page as usize].byte_offset() + (index_in_page * element_size)
    }
}

pub struct GPUDataUpdaterCopy {
    src_allocation: TransientBufferAllocation,
    static_buffer_offset: u64,
}

pub struct GPUDataUpdaterBuilder {
    allocator: TransientBufferAllocator,
    upload_jobs: Vec<GPUDataUpdaterCopy>,
}

impl GPUDataUpdaterBuilder {
    pub fn new(allocator: TransientBufferAllocator) -> Self {
        Self {
            allocator,
            upload_jobs: Vec::new(),
        }
    }

    pub fn add_update_jobs<T>(&mut self, data: &[T], dst_offset: u64) {
        let transient_allocation = self
            .allocator
            .copy_data_slice(data, ResourceUsage::AS_SHADER_RESOURCE);

        self.upload_jobs.push(GPUDataUpdaterCopy {
            src_allocation: transient_allocation,
            static_buffer_offset: dst_offset,
            });
    }

    pub fn job_blocks(self) -> Vec<GPUDataUpdaterCopy> {
        self.upload_jobs
    }
}
