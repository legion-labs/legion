use std::{
    alloc::Layout,
    sync::{Arc, Mutex, RwLock},
};

use lgn_graphics_api::{
    BarrierQueueTransition, Buffer, BufferAllocation, BufferBarrier, BufferCopy, BufferDef,
    BufferView, BufferViewDef, DeviceContext, IndexBufferBinding, IndexType, MemoryAllocation,
    MemoryAllocationDef, MemoryPagesAllocation, MemoryUsage, PagedBufferAllocation,
    ResourceCreation, ResourceState, ResourceUsage, Semaphore, VertexBufferBinding,
};
use lgn_tracing::span_fn;

use super::{Range, RangeAllocator, SparseBindingManager, TransientPagedBuffer};
use crate::RenderContext;

pub struct UnifiedStaticBuffer {
    buffer: Buffer,
    read_only_view: BufferView,
    _allocation: Option<MemoryAllocation>,
    allocator: UnifiedStaticBufferAllocator,
}

impl UnifiedStaticBuffer {
    pub fn new(
        device_context: &DeviceContext,
        virtual_buffer_size: u64,
        sparse_binding: bool,
    ) -> Self {
        let mut creation_flags = ResourceCreation::empty();
        if sparse_binding {
            creation_flags |= ResourceCreation::SPARSE_BINDING;
        }
        let buffer_def = BufferDef {
            name: "UnifiedStaticBuffer".to_string(),
            size: virtual_buffer_size,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_TRANSFERABLE
                | ResourceUsage::AS_VERTEX_BUFFER
                | ResourceUsage::AS_INDEX_BUFFER,
            creation_flags,
        };

        let buffer = device_context.create_buffer(&buffer_def);
        let required_alignment = buffer.required_alignment();

        assert!(virtual_buffer_size % required_alignment as u64 == 0);

        let ro_view_def = BufferViewDef::as_byte_address_buffer(buffer.definition(), true);
        let read_only_view = BufferView::from_buffer(&buffer, &ro_view_def);

        let (allocation, binding_manager) = if sparse_binding {
            (None, Some(SparseBindingManager::new()))
        } else {
            let alloc_def = MemoryAllocationDef {
                memory_usage: MemoryUsage::GpuOnly,
                always_mapped: false,
            };

            (
                Some(MemoryAllocation::from_buffer(
                    device_context,
                    &buffer,
                    &alloc_def,
                )),
                None,
            )
        };

        Self {
            buffer: buffer.clone(),
            _allocation: allocation,
            read_only_view,
            allocator: UnifiedStaticBufferAllocator::new(
                &buffer,
                virtual_buffer_size,
                binding_manager,
                sparse_binding,
                required_alignment,
            ),
        }
    }

    pub fn allocator(&self) -> &UnifiedStaticBufferAllocator {
        &self.allocator
    }

    pub fn read_only_view(&self) -> BufferView {
        self.read_only_view.clone()
    }

    pub fn index_buffer_binding(&self) -> IndexBufferBinding<'_> {
        IndexBufferBinding {
            buffer: &self.buffer,
            byte_offset: 0,
            index_type: IndexType::Uint16,
        }
    }
}

pub(crate) struct StaticBufferAllocation {
    allocator: UnifiedStaticBufferAllocator,
    allocation: Option<PagedBufferAllocation>,
}

impl Drop for StaticBufferAllocation {
    fn drop(&mut self) {
        self.allocator.free_segment(self.allocation.take().unwrap());
    }
}

impl StaticBufferAllocation {
    pub fn offset(&self) -> u64 {
        self.allocation.as_ref().unwrap().byte_offset()
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding<'_> {
        self.allocation.as_ref().unwrap().vertex_buffer_binding()
    }

    pub fn create_structured_buffer_view(&self, struct_size: u64, read_only: bool) -> BufferView {
        self.allocation
            .as_ref()
            .unwrap()
            .create_structured_buffer_view(struct_size, read_only)
    }
}

pub(crate) struct UnifiedStaticBufferAllocatorInner {
    buffer: Buffer,
    segment_allocator: RangeAllocator,
    binding_manager: Option<SparseBindingManager>,
    sparse_binding: bool,
    page_size: u64,
    job_blocks: Vec<UniformGPUDataUploadJobBlock>,
}

#[derive(Clone)]
pub struct UnifiedStaticBufferAllocator {
    inner: Arc<Mutex<UnifiedStaticBufferAllocatorInner>>,
}

impl UnifiedStaticBufferAllocator {
    pub fn new(
        buffer: &Buffer,
        virtual_buffer_size: u64,
        binding_manager: Option<SparseBindingManager>,
        sparse_binding: bool,
        required_alignment: u64,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(UnifiedStaticBufferAllocatorInner {
                buffer: buffer.clone(),
                segment_allocator: RangeAllocator::new(virtual_buffer_size),
                binding_manager,
                sparse_binding,
                page_size: required_alignment,
                job_blocks: Vec::new(),
            })),
        }
    }

    pub(crate) fn allocate_segment(&self, segment_size: u64) -> StaticBufferAllocation {
        let inner = &mut *self.inner.lock().unwrap();

        let page_size = inner.page_size;
        let alloc_size = lgn_utils::memory::round_size_up_to_alignment_u64(segment_size, page_size);
        let page_count = alloc_size / page_size;

        if segment_size != alloc_size {
            // TODO(vdbdd): use warn instead
            println!( "UnifiedStaticBufferAllocator: the segment required size ({} bytes) is less than the allocated size ({} bytes). {} bytes of memory will be wasted", segment_size, alloc_size, alloc_size-segment_size  );
        }

        let location = inner.segment_allocator.allocate(alloc_size).unwrap();

        let allocation = if inner.sparse_binding {
            MemoryPagesAllocation::for_sparse_buffer(
                inner.buffer.device_context(),
                &inner.buffer,
                page_count,
            )
        } else {
            MemoryPagesAllocation::empty_allocation(inner.buffer.device_context())
        };

        let allocation = PagedBufferAllocation {
            buffer: inner.buffer.clone(),
            memory: allocation,
            byte_offset: location.begin(),
            size: location.size(),
        };

        if let Some(binding_manager) = &mut inner.binding_manager {
            binding_manager.add_sparse_binding(allocation.clone());
        }

        StaticBufferAllocation {
            allocator: self.clone(),
            allocation: Some(allocation),
        }
    }

    fn free_segment(&self, segment: PagedBufferAllocation) {
        let inner = &mut *self.inner.lock().unwrap();

        inner
            .segment_allocator
            .free(Range::from_begin_size(segment.byte_offset, segment.size));

        if let Some(binding_manager) = &mut inner.binding_manager {
            binding_manager.add_sparse_unbinding(segment);
        }
    }

    pub fn add_update_job_block(&self, job_blocks: &mut Vec<UniformGPUDataUploadJobBlock>) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.job_blocks.append(job_blocks);
    }

    #[span_fn]
    pub(crate) fn flush_updater(
        &self,
        prev_frame_semaphore: &Semaphore,
        unbind_semaphore: &Semaphore,
        bind_semaphore: &Semaphore,
        render_context: &RenderContext<'_>,
    ) {
        let inner = &mut *self.inner.lock().unwrap();

        let mut last_semaphore = None;
        let graphics_queue = render_context.graphics_queue();
        if let Some(binding_manager) = &mut inner.binding_manager {
            last_semaphore = Some(binding_manager.commit_sparse_bindings(
                &graphics_queue,
                prev_frame_semaphore,
                unbind_semaphore,
                bind_semaphore,
            ));
        }

        let cmd_buffer = render_context.alloc_command_buffer();

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
                &job.upload_allocation.buffer,
                &inner.buffer,
                &job.upload_jobs,
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

        let mut wait_sems = Vec::new();
        if let Some(wait_sem) = last_semaphore {
            if wait_sem.signal_available() {
                wait_sems.push(wait_sem);
                wait_sem.set_signal_available(false);
            }
        }

        graphics_queue.submit(&mut [cmd_buffer.finalize()], &wait_sems, &[], None);
    }
}

pub struct UniformGPUData<T> {
    allocated_pages: RwLock<Vec<StaticBufferAllocation>>,
    elements_per_page: u64,
    marker: std::marker::PhantomData<T>,
}

impl<T> UniformGPUData<T> {
    pub fn new(allocator: Option<&UnifiedStaticBufferAllocator>, elements_per_page: u64) -> Self {
        let mut allocated_pages = Vec::new();
        if let Some(allocator) = allocator {
            let page_size = elements_per_page * std::mem::size_of::<T>() as u64;
            let page = allocator.allocate_segment(page_size);
            allocated_pages.push(page);
        }
        Self {
            allocated_pages: RwLock::new(allocated_pages),
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
                return page_read_access[index_of_page as usize].offset()
                    + (index_in_page * element_size);
            }
        }

        let mut page_write_access = self.allocated_pages.write().unwrap();

        while (page_write_access.len() as u64) < required_pages {
            let segment_size = elements_per_page * std::mem::size_of::<T>() as u64;
            page_write_access.push(allocator.allocate_segment(segment_size));
        }

        page_write_access[index_of_page as usize].offset() + (index_in_page * element_size)
    }

    pub fn create_structured_buffer_view(&self, struct_size: u64) -> BufferView {
        let page_read_access = self.allocated_pages.read().unwrap();

        assert!(!page_read_access.is_empty());
        page_read_access[0].create_structured_buffer_view(struct_size, true)
    }

    pub fn offset(&self) -> u64 {
        let page_read_access = self.allocated_pages.read().unwrap();

        assert!(!page_read_access.is_empty());
        page_read_access[0].offset()
    }
}

pub struct UniformGPUDataUploadJobBlock {
    upload_allocation: BufferAllocation,
    upload_jobs: Vec<BufferCopy>,
    offset: u64,
}

impl UniformGPUDataUploadJobBlock {
    fn new(upload_allocation: BufferAllocation) -> Self {
        Self {
            upload_allocation,
            upload_jobs: Vec::new(),
            offset: 0,
        }
    }

    fn add_update_jobs<T>(&mut self, data: &[T], dst_offset: u64) -> bool {
        let upload_size_in_bytes = lgn_utils::memory::slice_size_in_bytes(data) as u64;
        if self.offset + upload_size_in_bytes <= self.upload_allocation.size() {
            let src = data.as_ptr().cast::<u8>();
            let upload_offset = self.upload_allocation.byte_offset() + self.offset;
            {
                #[allow(unsafe_code)]
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        src,
                        self.upload_allocation
                            .memory
                            .mapped_ptr()
                            .add(upload_offset as usize),
                        upload_size_in_bytes as usize,
                    );
                }
            }

            self.upload_jobs.push(BufferCopy {
                src_offset: upload_offset,
                dst_offset,
                size: upload_size_in_bytes,
            });
            self.offset += upload_size_in_bytes;

            true
        } else {
            false
        }
    }
}

pub struct UniformGPUDataUpdater {
    paged_buffer: TransientPagedBuffer,
    job_blocks: Vec<UniformGPUDataUploadJobBlock>,
    block_size: u64,
}

impl UniformGPUDataUpdater {
    pub fn new(paged_buffer: TransientPagedBuffer, block_size: u64) -> Self {
        Self {
            paged_buffer,
            job_blocks: Vec::new(),
            block_size,
        }
    }

    pub fn add_update_jobs<T>(&mut self, data: &[T], dst_offset: u64) {
        let upload_size_in_bytes = lgn_utils::memory::slice_size_in_bytes(data) as u64;
        assert!(dst_offset != u64::from(u32::MAX));

        while self.job_blocks.is_empty()
            || !self
                .job_blocks
                .last_mut()
                .unwrap()
                .add_update_jobs(data, dst_offset)
        {
            let data_layout = Layout::from_size_align(
                std::cmp::max(self.block_size as usize, upload_size_in_bytes as usize),
                std::mem::align_of::<T>(),
            )
            .unwrap();

            self.job_blocks.push(UniformGPUDataUploadJobBlock::new(
                self.paged_buffer.allocate_page(data_layout),
            ));
        }
    }

    pub fn job_blocks(&mut self) -> &mut Vec<UniformGPUDataUploadJobBlock> {
        &mut self.job_blocks
    }
}
