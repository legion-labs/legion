use std::sync::{Arc, Mutex, RwLock};

use lgn_graphics_api::{
    Buffer, BufferCreateFlags, BufferDef, BufferView, BufferViewDef, DeviceContext,
    IndexBufferBinding, IndexType, MemoryUsage, ResourceUsage, VertexBufferBinding,
};

use super::{Range, RangeAllocator};
use crate::core::{
    GpuUploadManager, RenderCommand, RenderResources, UploadGPUBuffer, UploadGPUResource,
};

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

        let buffer = device_context.create_buffer(
            BufferDef {
                size: buffer_size,
                usage_flags: ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_TRANSFERABLE
                    | ResourceUsage::AS_VERTEX_BUFFER
                    | ResourceUsage::AS_INDEX_BUFFER,
                create_flags: BufferCreateFlags::empty(),
                memory_usage: MemoryUsage::GpuOnly,
                always_mapped: false,
            },
            "UnifiedStaticBuffer",
        );

        let read_only_view =
            buffer.create_view(BufferViewDef::as_byte_address_buffer(element_count, true));

        let allocator = UnifiedStaticBufferAllocator::new(&buffer, buffer_size);

        Self {
            buffer,
            read_only_view,
            allocator,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
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
    _allocation: StaticBufferAllocation,
    buffer_view: BufferView,
}

impl StaticBufferView {
    fn new(allocation: &StaticBufferAllocation, view_definition: BufferViewDef) -> Self {
        let buffer_view = allocation.inner.buffer.create_view(view_definition);
        Self {
            _allocation: allocation.clone(),
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
    // job_blocks: Vec<GPUDataUpdaterCopy>,
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
                // job_blocks: Vec::new(),
            })),
        }
    }

    pub(crate) fn allocate(&self, required_size: u64) -> StaticBufferAllocation {
        let inner = &mut *self.inner.lock().unwrap();

        let alloc_size =
            lgn_utils::memory::round_size_up_to_alignment_u64(required_size, PAGE_SIZE);

        if required_size != alloc_size {
            // TODO(vdbdd): use warn instead
            println!( "UnifiedStaticBufferAllocator: the segment required size ({} bytes) is less than the allocated size ({} bytes). {} bytes of memory will be wasted", required_size, alloc_size, alloc_size-required_size  );
        }

        let range = inner.segment_allocator.allocate(alloc_size).unwrap();

        StaticBufferAllocation::new(self, &inner.buffer, range)
    }

    fn free(&self, range: Range) {
        let inner = &mut *self.inner.lock().unwrap();
        inner.segment_allocator.free(range);
    }
}

pub struct UniformGPUData<T> {
    gpu_allocator: UnifiedStaticBufferAllocator,
    allocated_pages: RwLock<Vec<StaticBufferAllocation>>,
    elements_per_page: u64,
    marker: std::marker::PhantomData<T>,
}

impl<T> UniformGPUData<T> {
    pub fn new(gpu_allocator: &UnifiedStaticBufferAllocator, elements_per_page: u64) -> Self {
        Self {
            gpu_allocator: gpu_allocator.clone(),
            allocated_pages: RwLock::new(Vec::new()),
            elements_per_page,
            marker: ::std::marker::PhantomData,
        }
    }

    pub fn ensure_index_allocated(&self, index: u32) -> u64 {
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
            page_write_access.push(self.gpu_allocator.allocate(segment_size));
        }

        page_write_access[index_of_page as usize].byte_offset() + (index_in_page * element_size)
    }
}

#[derive(Debug)]
pub struct UpdateUnifiedStaticBufferCommand {
    pub src_buffer: Vec<u8>,
    pub dst_offset: u64,
}

impl RenderCommand for UpdateUnifiedStaticBufferCommand {
    fn execute(self, render_resources: &RenderResources) {
        let mut upload_manager = render_resources.get_mut::<GpuUploadManager>();
        let unified_static_buffer = render_resources.get::<UnifiedStaticBuffer>();
        upload_manager.push(UploadGPUResource::Buffer(UploadGPUBuffer {
            src_data: self.src_buffer,
            dst_buffer: unified_static_buffer.buffer().clone(),
            dst_offset: self.dst_offset,
        }));
    }
}
