use std::sync::{Arc, Mutex};

use graphics_api::{
    Buffer, BufferDef, DeviceContext, MemoryPagesAllocation, Queue, QueueType, ResourceCreation,
    ResourceUsage, Semaphore,
};

use super::{Range, RangeAllocator};

pub(crate) struct SparseBindingManager {
    sparse_buffer_bindings: Vec<MemoryPagesAllocation>,
    sparse_buffer_unbindings: Vec<MemoryPagesAllocation>,
}

impl SparseBindingManager {
    pub fn new() -> Self {
        Self {
            sparse_buffer_bindings: Vec::new(),
            sparse_buffer_unbindings: Vec::new(),
        }
    }

    pub fn add_sparse_binding(&mut self, binding: &MemoryPagesAllocation) {
        self.sparse_buffer_bindings.push(binding.clone());
    }

    pub fn add_sparse_unbinding(&mut self, unbinding: &MemoryPagesAllocation) {
        self.sparse_buffer_unbindings.push(unbinding.clone());
    }

    pub fn commmit_sparse_bindings(
        &mut self,
        queue: &Queue,
        prev_frame_semaphore: Semaphore,
        unbind_semaphore: Semaphore,
        bind_semaphore: Semaphore,
    ) -> Option<Semaphore> {
        let result = queue.commmit_sparse_bindings(
            prev_frame_semaphore,
            &self.sparse_buffer_unbindings,
            unbind_semaphore,
            &self.sparse_buffer_bindings,
            bind_semaphore,
        );

        self.sparse_buffer_unbindings.clear();
        self.sparse_buffer_bindings.clear();

        result
    }
}

enum StaticResourceType {
    VertexBuffer = 0,
    IndexBuffer = 1,
}

pub(crate) struct UnifiedStaticBufferInner {
    buffer: Buffer,
    range_allocator: RangeAllocator,
    binding_manager: SparseBindingManager,
    page_size: u64,
}

#[derive(Clone)]
pub struct UnifiedStaticBuffer {
    inner: Arc<Mutex<UnifiedStaticBufferInner>>,
}

impl UnifiedStaticBuffer {
    pub fn new(device_context: &DeviceContext, virtual_buffer_size: u64) -> Self {
        let buffer_def = BufferDef {
            size: virtual_buffer_size,
            queue_type: QueueType::Graphics,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE,
            creation_flags: ResourceCreation::SPARSE_BINDING,
        };

        let buffer = device_context.create_buffer(&buffer_def);
        let required_alignment = buffer.required_alignment();

        assert!(virtual_buffer_size % required_alignment == 0);

        Self {
            inner: Arc::new(Mutex::new(UnifiedStaticBufferInner {
                buffer,
                range_allocator: RangeAllocator::new(virtual_buffer_size / required_alignment),
                binding_manager: SparseBindingManager::new(),
                page_size: required_alignment,
            })),
        }
    }

    pub fn commmit_sparse_bindings(
        &mut self,
        queue: &Queue,
        prev_frame_semaphore: Semaphore,
        unbind_semaphore: Semaphore,
        bind_semaphore: Semaphore,
    ) -> Option<Semaphore> {
        let inner = &mut *self.inner.lock().unwrap();

        inner.binding_manager.commmit_sparse_bindings(
            queue,
            prev_frame_semaphore,
            unbind_semaphore,
            bind_semaphore,
        )
    }

    fn allocate_block(&self, block_size: u64) -> (Range, MemoryPagesAllocation) {
        let inner = &mut *self.inner.lock().unwrap();

        let page_size = inner.page_size;
        let page_count =
            legion_utils::memory::round_size_up_to_alignment_u64(block_size, page_size) / page_size;

        let block = inner.range_allocator.allocate(page_count).unwrap();
        let allocation = MemoryPagesAllocation::for_sparse_buffer(
            inner.buffer.device_context(),
            &inner.buffer,
            block.first * page_size,
            page_count,
        );

        inner.binding_manager.add_sparse_binding(&allocation);

        (block, allocation)
    }

    fn free_block(&self, block: Range, allocation: &MemoryPagesAllocation) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.binding_manager.add_sparse_unbinding(allocation);

        inner.range_allocator.free(block);
    }
}

pub(crate) struct StaticBufferBlock {
    block_range: Range,
    block_allocation: MemoryPagesAllocation,
    range_allocator: Option<RangeAllocator>,
}
