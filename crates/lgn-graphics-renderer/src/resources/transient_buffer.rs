use std::{
    ptr::NonNull,
    sync::{Arc, Mutex},
};

use lgn_core::Handle;
use lgn_graphics_api::{
    Buffer, BufferCreateFlags, BufferDef, BufferViewDef, DeviceContext, IndexBufferBinding,
    IndexType, MemoryUsage, ResourceUsage, TransientBufferView, VertexBufferBinding,
};

use super::GpuSafePool;

const TRANSIENT_BUFFER_RESOURCE_USAGE: ResourceUsage = ResourceUsage::from_bits_truncate(
    ResourceUsage::AS_SHADER_RESOURCE.bits()
        | ResourceUsage::AS_UNORDERED_ACCESS.bits()
        | ResourceUsage::AS_CONST_BUFFER.bits()
        | ResourceUsage::AS_VERTEX_BUFFER.bits()
        | ResourceUsage::AS_INDEX_BUFFER.bits(),
);

const TRANSIENT_BUFFER_BLOCK_SIZE: u64 = 64 * 1024;

pub struct TransientBufferAllocation {
    buffer: NonNull<Buffer>,
    byte_offset: u64,
    resource_usage: ResourceUsage,
    _size: u64,
}

impl TransientBufferAllocation {
    pub fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    #[allow(unsafe_code)]
    pub fn buffer(&self) -> &Buffer {
        unsafe { self.buffer.as_ref() }
    }

    #[allow(unsafe_code)]
    pub fn mapped_ptr(&self) -> *mut u8 {
        unsafe {
            self.buffer
                .as_ref()
                .mapped_ptr()
                .add(self.byte_offset as usize)
        }
    }

    #[allow(unsafe_code)]
    pub fn mapped_ptr_typed<T: Sized>(&self) -> *mut T {
        let ptr = self.mapped_ptr();
        ptr.cast::<T>()
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding {
        assert!(self
            .resource_usage
            .intersects(ResourceUsage::AS_VERTEX_BUFFER));
        VertexBufferBinding::new(self.buffer(), self.byte_offset)
    }

    pub fn index_buffer_binding(&self, index_type: IndexType) -> IndexBufferBinding {
        assert!(self
            .resource_usage
            .intersects(ResourceUsage::AS_INDEX_BUFFER));
        IndexBufferBinding::new(self.buffer(), self.byte_offset, index_type)
    }

    pub fn to_buffer_view(&self, view_def: BufferViewDef) -> TransientBufferView {
        match view_def.gpu_view_type {
            lgn_graphics_api::GPUViewType::ConstantBuffer => assert!(self
                .resource_usage
                .intersects(ResourceUsage::AS_CONST_BUFFER)),
            lgn_graphics_api::GPUViewType::ShaderResource => assert!(self
                .resource_usage
                .intersects(ResourceUsage::AS_SHADER_RESOURCE)),
            lgn_graphics_api::GPUViewType::UnorderedAccess => assert!(self
                .resource_usage
                .intersects(ResourceUsage::AS_UNORDERED_ACCESS)),
            lgn_graphics_api::GPUViewType::RenderTarget
            | lgn_graphics_api::GPUViewType::DepthStencil => panic!(),
        }

        let view_def = BufferViewDef {
            byte_offset: self.byte_offset,
            ..view_def
        };
        self.buffer().create_transient_view(view_def)
    }
}

fn compute_required_alignment(
    device_context: &DeviceContext,
    resource_usage: ResourceUsage,
) -> u32 {
    let resource_usage = if resource_usage.is_empty() {
        TRANSIENT_BUFFER_RESOURCE_USAGE
    } else {
        resource_usage
    };

    let required_alignment = if resource_usage.intersects(ResourceUsage::AS_CONST_BUFFER) {
        device_context
            .device_info()
            .min_uniform_buffer_offset_alignment
    } else {
        device_context
            .device_info()
            .min_storage_buffer_offset_alignment
    };

    required_alignment
}

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

        let buffer = device_context.create_buffer(
            BufferDef {
                size: required_size,
                usage_flags: TRANSIENT_BUFFER_RESOURCE_USAGE,
                create_flags: BufferCreateFlags::empty(),
                memory_usage: MemoryUsage::CpuToGpu,
                always_mapped: true,
            },
            "PageHeap",
        );

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

    pub fn size(&self, required_alignment: u32) -> u64 {
        let aligned_offset = lgn_utils::memory::round_size_up_to_alignment_u64(
            self.byte_offset,
            u64::from(required_alignment),
        );
        self.capacity - aligned_offset
    }

    fn allocate(
        &mut self,
        required_size: u64,
        resource_usage: ResourceUsage,
    ) -> Option<TransientBufferAllocation> {
        assert_eq!(
            ResourceUsage::empty(),
            resource_usage & TRANSIENT_BUFFER_RESOURCE_USAGE.complement()
        );

        let resource_usage = if resource_usage.is_empty() {
            TRANSIENT_BUFFER_RESOURCE_USAGE
        } else {
            resource_usage
        };

        let required_alignment = compute_required_alignment(&self.device_context, resource_usage);

        let aligned_offset = lgn_utils::memory::round_size_up_to_alignment_u64(
            self.byte_offset,
            u64::from(required_alignment),
        );
        let next_offset = aligned_offset + required_size;

        if next_offset > self.capacity {
            None
        } else {
            self.byte_offset = next_offset;

            Some(TransientBufferAllocation {
                buffer: NonNull::from(&self.buffer),
                byte_offset: aligned_offset,
                resource_usage,
                _size: required_size,
            })
        }
    }
}

struct Inner {
    device_context: DeviceContext,
    transient_buffers: GpuSafePool<TransientBuffer>,
    frame_pool: Vec<Handle<TransientBuffer>>,
}

#[derive(Clone)]
pub struct TransientBufferManager {
    inner: Arc<Mutex<Inner>>,
}

impl TransientBufferManager {
    pub fn new(device_context: &DeviceContext, num_cpu_frames: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                device_context: device_context.clone(),
                transient_buffers: GpuSafePool::new(num_cpu_frames),
                frame_pool: Vec::new(),
            })),
        }
    }

    pub fn begin_frame(&mut self, frame_index: usize) {
        let mut inner = self.inner.lock().unwrap();

        assert!(inner.frame_pool.is_empty());

        inner
            .transient_buffers
            .begin_frame(frame_index, TransientBuffer::begin_frame);
    }

    pub fn end_frame(&mut self, frame_index: usize) {
        let mut inner = self.inner.lock().unwrap();

        let mut frame_pool = std::mem::take(&mut inner.frame_pool);

        frame_pool.drain(..).for_each(|handle| {
            inner.transient_buffers.release(handle);
        });

        inner.transient_buffers.end_frame(frame_index, |_| ());
    }

    fn acquire_page(
        &self,
        min_page_size: u64,
        resource_usage: ResourceUsage,
    ) -> Handle<TransientBuffer> {
        let inner = &mut *self.inner.lock().unwrap();

        let required_alignment = compute_required_alignment(&inner.device_context, resource_usage);

        for (i, handle) in inner.frame_pool.iter().enumerate() {
            if min_page_size <= handle.size(required_alignment) {
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
    paged_buffer: TransientBufferManager,
    transient_buffer: Handle<TransientBuffer>,
}

impl TransientBufferAllocator {
    pub fn new(paged_buffer: &TransientBufferManager, min_alloc_size: u64) -> Self {
        let allocation = paged_buffer.acquire_page(min_alloc_size, ResourceUsage::empty());
        Self {
            paged_buffer: paged_buffer.clone(),
            transient_buffer: allocation,
        }
    }

    pub fn allocate(&mut self, size: u64) -> TransientBufferAllocation {
        self.allocate_with_usage(size, ResourceUsage::empty())
    }

    pub fn allocate_with_usage(
        &mut self,
        size: u64,
        resource_usage: ResourceUsage,
    ) -> TransientBufferAllocation {
        self.allocate_inner(size, resource_usage)
    }

    pub fn allocate_from_view(&mut self, view_def: BufferViewDef) -> TransientBufferAllocation {
        let size = view_def.element_count * view_def.element_size;
        let resource_usage = match view_def.gpu_view_type {
            lgn_graphics_api::GPUViewType::ConstantBuffer => ResourceUsage::AS_CONST_BUFFER,
            lgn_graphics_api::GPUViewType::ShaderResource => ResourceUsage::AS_SHADER_RESOURCE,
            lgn_graphics_api::GPUViewType::UnorderedAccess => ResourceUsage::AS_UNORDERED_ACCESS,
            lgn_graphics_api::GPUViewType::RenderTarget
            | lgn_graphics_api::GPUViewType::DepthStencil => unreachable!(),
        };
        self.allocate_inner(size, resource_usage)
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
        let src_size = std::mem::size_of_val(data);
        let src = data.as_ptr().cast::<u8>();
        let dst = self.allocate_inner(src_size as u64, resource_usage);

        #[allow(unsafe_code)]
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst.mapped_ptr(), src_size);
        }

        dst
    }

    fn allocate_inner(
        &mut self,
        size: u64,
        resource_usage: ResourceUsage,
    ) -> TransientBufferAllocation {
        let mut allocation = self.transient_buffer.allocate(size, resource_usage);

        while allocation.is_none() {
            self.paged_buffer
                .release_page(self.transient_buffer.transfer());
            self.transient_buffer = self.paged_buffer.acquire_page(size, resource_usage);
            allocation = self.transient_buffer.allocate(size, resource_usage);
        }

        allocation.unwrap()
    }
}

impl Drop for TransientBufferAllocator {
    fn drop(&mut self) {
        self.paged_buffer
            .release_page(self.transient_buffer.transfer());
    }
}
