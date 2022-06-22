use std::sync::{Arc, Mutex, RwLock};

use lgn_graphics_api::{
    Buffer, BufferCreateFlags, BufferDef, BufferView, BufferViewDef, DeviceContext, GPUViewType,
    IndexBufferBinding, IndexType, MemoryUsage, ResourceUsage, VertexBufferBinding,
};
use lgn_tracing::warn;

use super::{Range, RangeAllocator};
use crate::core::{
    GpuUploadManager, RenderCommand, RenderResources, UploadGPUBuffer, UploadGPUResource,
};

const STATIC_BUFFER_RESOURCE_USAGE: ResourceUsage = ResourceUsage::from_bits_truncate(
    ResourceUsage::AS_SHADER_RESOURCE.bits()
        | ResourceUsage::AS_UNORDERED_ACCESS.bits()
        | ResourceUsage::AS_CONST_BUFFER.bits()
        | ResourceUsage::AS_VERTEX_BUFFER.bits()
        | ResourceUsage::AS_INDEX_BUFFER.bits()
        | ResourceUsage::AS_TRANSFERABLE.bits(),
);

struct Inner {
    buffer: Buffer,
    read_only_view: BufferView,
    required_alignment: u32,
    allocator: Mutex<RangeAllocator>,
}

#[derive(Clone)]
pub struct UnifiedStaticBuffer {
    inner: Arc<Inner>,
}

impl UnifiedStaticBuffer {
    pub fn new(device_context: &DeviceContext, virtual_buffer_size: u64) -> Self {
        let element_size = std::mem::size_of::<u32>() as u64;
        let element_count = virtual_buffer_size / element_size;
        let buffer_size = element_count * element_size;

        let buffer = device_context.create_buffer(
            BufferDef {
                size: buffer_size,
                usage_flags: STATIC_BUFFER_RESOURCE_USAGE,
                create_flags: BufferCreateFlags::empty(),
                memory_usage: MemoryUsage::GpuOnly,
                always_mapped: false,
            },
            "UnifiedStaticBuffer",
        );

        let read_only_view =
            buffer.create_view(BufferViewDef::as_byte_address_buffer(element_count, true));

        let required_alignment = std::cmp::max(
            device_context
                .device_info()
                .min_uniform_buffer_offset_alignment,
            device_context
                .device_info()
                .min_storage_buffer_offset_alignment,
        );

        let allocator = Mutex::new(RangeAllocator::new(buffer.definition().size));

        Self {
            inner: Arc::new(Inner {
                buffer,
                read_only_view,
                required_alignment,
                allocator,
            }),
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.inner.buffer
    }

    pub fn read_only_view(&self) -> &BufferView {
        &self.inner.read_only_view
    }

    pub fn index_buffer_binding(&self) -> IndexBufferBinding {
        IndexBufferBinding::new(&self.inner.buffer, 0, IndexType::Uint16)
    }

    pub fn allocate(
        &self,
        required_size: u64,
        resource_usage: ResourceUsage,
    ) -> StaticBufferAllocation {
        assert_eq!(
            ResourceUsage::empty(),
            resource_usage & STATIC_BUFFER_RESOURCE_USAGE.complement()
        );

        let resource_usage = if resource_usage.is_empty() {
            STATIC_BUFFER_RESOURCE_USAGE
        } else {
            resource_usage
        };

        let alloc_size = lgn_utils::memory::round_size_up_to_alignment_u64(
            required_size,
            u64::from(self.inner.required_alignment),
        );

        if required_size != alloc_size {
            warn!( "UnifiedStaticBuffer: the segment required size ({} bytes) is less than the allocated size ({} bytes). {} bytes of memory will be wasted", required_size, alloc_size, alloc_size-required_size  );
        }

        let allocator = &mut *self.inner.allocator.lock().unwrap();

        let alloc_range = allocator.allocate(alloc_size).unwrap();

        assert_eq!(
            alloc_range.begin() % u64::from(self.inner.required_alignment),
            0
        );
        assert!(alloc_range.size() >= required_size);

        StaticBufferAllocation::new(self, alloc_range, resource_usage)
    }

    fn free(&self, range: Range) {
        let allocator = &mut *self.inner.allocator.lock().unwrap();
        allocator.free(range);
    }
}

pub struct StaticBufferView {
    _allocation: StaticBufferAllocation,
    buffer_view: BufferView,
}

impl StaticBufferView {
    fn new(allocation: &StaticBufferAllocation, view_definition: BufferViewDef) -> Self {
        Self {
            _allocation: allocation.clone(),
            buffer_view: allocation.buffer().create_view(view_definition),
        }
    }

    pub fn buffer_view(&self) -> &BufferView {
        &self.buffer_view
    }
}

struct StaticBufferAllocationInner {
    gpu_heap: UnifiedStaticBuffer,
    alloc_range: Range,
    resource_usage: ResourceUsage,
}

#[derive(Clone)]
pub struct StaticBufferAllocation {
    inner: Arc<StaticBufferAllocationInner>,
}

impl StaticBufferAllocation {
    fn new(
        gpu_heap: &UnifiedStaticBuffer,
        alloc_range: Range,
        resource_usage: ResourceUsage,
    ) -> Self {
        Self {
            inner: Arc::new(StaticBufferAllocationInner {
                gpu_heap: gpu_heap.clone(),
                alloc_range,
                resource_usage,
            }),
        }
    }

    pub fn buffer(&self) -> &Buffer {
        self.inner.gpu_heap.buffer()
    }

    pub fn byte_offset(&self) -> u64 {
        self.inner.alloc_range.begin()
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding {
        assert!(self
            .inner
            .resource_usage
            .intersects(ResourceUsage::AS_VERTEX_BUFFER));
        VertexBufferBinding::new(self.buffer(), self.byte_offset())
    }

    #[allow(dead_code)]
    pub fn index_buffer_binding(&self, index_type: IndexType) -> IndexBufferBinding {
        assert!(self
            .inner
            .resource_usage
            .intersects(ResourceUsage::AS_INDEX_BUFFER));
        IndexBufferBinding::new(self.buffer(), self.byte_offset(), index_type)
    }

    pub fn create_view(&self, view_definition: BufferViewDef) -> StaticBufferView {
        match view_definition.gpu_view_type {
            GPUViewType::ConstantBuffer => assert!(self
                .inner
                .resource_usage
                .intersects(ResourceUsage::AS_CONST_BUFFER)),
            GPUViewType::ShaderResource => assert!(self
                .inner
                .resource_usage
                .intersects(ResourceUsage::AS_SHADER_RESOURCE)),
            GPUViewType::UnorderedAccess => assert!(self
                .inner
                .resource_usage
                .intersects(ResourceUsage::AS_UNORDERED_ACCESS)),
            GPUViewType::RenderTarget | GPUViewType::DepthStencil => panic!(),
        }

        let view_definition = BufferViewDef {
            byte_offset: self.byte_offset(),
            ..view_definition
        };

        StaticBufferView::new(self, view_definition)
    }
}

impl Drop for StaticBufferAllocationInner {
    fn drop(&mut self) {
        self.gpu_heap.free(self.alloc_range);
    }
}

pub struct UniformGPUData<T> {
    gpu_allocator: UnifiedStaticBuffer,
    allocated_pages: RwLock<Vec<StaticBufferAllocation>>,
    elements_per_page: u64,
    marker: std::marker::PhantomData<T>,
}

impl<T> UniformGPUData<T> {
    pub fn new(gpu_heap: &UnifiedStaticBuffer, elements_per_page: u64) -> Self {
        Self {
            gpu_allocator: gpu_heap.clone(),
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
            page_write_access.push(
                self.gpu_allocator
                    .allocate(segment_size, ResourceUsage::AS_SHADER_RESOURCE),
            );
        }

        page_write_access[index_of_page as usize].byte_offset() + (index_in_page * element_size)
    }
}

#[derive(Debug)]
pub struct UpdateUnifiedStaticBufferCommand {
    pub src_buffer: Vec<u8>,
    pub dst_offset: u64,
}

impl RenderCommand<RenderResources> for UpdateUnifiedStaticBufferCommand {
    fn execute(self, render_resources: &RenderResources) {
        let upload_manager = render_resources.get::<GpuUploadManager>();
        let unified_static_buffer = render_resources.get::<UnifiedStaticBuffer>();
        upload_manager.push(UploadGPUResource::Buffer(UploadGPUBuffer {
            src_data: self.src_buffer,
            dst_buffer: unified_static_buffer.buffer().clone(),
            dst_offset: self.dst_offset,
        }));
    }
}
