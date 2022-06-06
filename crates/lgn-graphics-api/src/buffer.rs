use crate::{
    backends::BackendBuffer, deferred_drop::Drc, BufferCreateFlags, BufferView, BufferViewDef,
    DeviceContext, MemoryUsage, ResourceUsage, TransientBufferView,
};

/// Used to create a `Buffer`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BufferDef {
    pub size: u64,
    pub usage_flags: ResourceUsage,
    pub create_flags: BufferCreateFlags,
    pub memory_usage: MemoryUsage,
    pub always_mapped: bool,
}

pub struct BufferMappingInfo<'a> {
    pub(crate) _buffer: &'a Buffer,
    pub(crate) data_ptr: *mut u8,
}

impl<'a> BufferMappingInfo<'a> {
    pub fn data_ptr(&self) -> *mut u8 {
        self.data_ptr
    }
}

impl Default for BufferDef {
    fn default() -> Self {
        Self {
            size: 0,
            usage_flags: ResourceUsage::empty(),
            create_flags: BufferCreateFlags::empty(),
            memory_usage: MemoryUsage::Unknown,
            always_mapped: false,
        }
    }
}

impl BufferDef {
    pub fn verify(&self) {
        assert_ne!(self.size, 0);
        assert!(!self
            .usage_flags
            .intersects(ResourceUsage::TEXTURE_ONLY_USAGE_FLAGS));
    }

    pub fn for_staging_buffer(size: usize, usage_flags: ResourceUsage) -> Self {
        Self {
            size: size as u64,
            usage_flags,
            create_flags: BufferCreateFlags::empty(),
            memory_usage: MemoryUsage::CpuToGpu,
            always_mapped: true,
        }
    }

    pub fn for_staging_buffer_data<T: Copy>(data: &[T], usage_flags: ResourceUsage) -> Self {
        Self::for_staging_buffer(lgn_utils::memory::slice_size_in_bytes(data), usage_flags)
    }

    pub fn for_staging_vertex_buffer(size: usize) -> Self {
        Self::for_staging_buffer(size, ResourceUsage::AS_VERTEX_BUFFER)
    }

    pub fn for_staging_vertex_buffer_data<T: Copy>(data: &[T]) -> Self {
        Self::for_staging_buffer_data(data, ResourceUsage::AS_VERTEX_BUFFER)
    }

    pub fn for_staging_index_buffer(size: usize) -> Self {
        Self::for_staging_buffer(size, ResourceUsage::AS_INDEX_BUFFER)
    }

    pub fn for_staging_index_buffer_data<T: Copy>(data: &[T]) -> Self {
        Self::for_staging_buffer_data(data, ResourceUsage::AS_INDEX_BUFFER)
    }
}

pub(crate) struct BufferInner {
    pub(crate) buffer_def: BufferDef,
    pub(crate) device_context: DeviceContext,
    pub(crate) buffer_id: u32,
    pub(crate) backend_buffer: BackendBuffer,
}

impl Drop for BufferInner {
    fn drop(&mut self) {
        self.backend_buffer
            .destroy(&self.device_context, &self.buffer_def);
    }
}

#[derive(Clone)]
pub struct Buffer {
    pub(crate) inner: Drc<BufferInner>,
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.inner.buffer_id == other.inner.buffer_id
    }
}

impl Buffer {
    pub fn new(device_context: &DeviceContext, buffer_def: BufferDef) -> Self {
        let (platform_buffer, buffer_id) = BackendBuffer::new(device_context, buffer_def);

        Self {
            inner: device_context.deferred_dropper().new_drc(BufferInner {
                device_context: device_context.clone(),
                buffer_def,
                buffer_id,
                backend_buffer: platform_buffer,
            }),
        }
    }

    pub fn definition(&self) -> &BufferDef {
        &self.inner.buffer_def
    }

    pub fn set_name<T: AsRef<str>>(&self, name: T) {
        self.inner.device_context.set_buffer_name(self, name);
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn create_view(&self, view_def: BufferViewDef) -> BufferView {
        BufferView::from_buffer(self, view_def)
    }

    pub fn create_transient_view(&self, view_def: BufferViewDef) -> TransientBufferView {
        TransientBufferView::from_buffer(self, view_def)
    }

    pub fn copy_to_host_visible_buffer<T: Copy>(&self, data: &[T]) {
        // Cannot check size of data == buffer because buffer size might be rounded up
        self.copy_to_host_visible_buffer_with_offset(data, 0);
    }

    pub fn copy_to_host_visible_buffer_with_offset<T: Copy>(
        &self,
        data: &[T],
        buffer_byte_offset: u64,
    ) {
        let data_size_in_bytes = lgn_utils::memory::slice_size_in_bytes(data) as u64;

        assert!(buffer_byte_offset + data_size_in_bytes <= self.definition().size as u64);

        let src = data.as_ptr().cast::<u8>();

        let required_alignment = std::mem::align_of::<T>();

        let mapping_info = self.map_buffer();

        #[allow(unsafe_code)]
        unsafe {
            let dst = mapping_info.data_ptr().add(buffer_byte_offset as usize);
            assert_eq!(((dst as usize) % required_alignment), 0);
            std::ptr::copy_nonoverlapping(src, dst, data_size_in_bytes as usize);
        }

        self.unmap_buffer();
    }

    pub fn map_buffer(&self) -> BufferMappingInfo<'_> {
        self.backend_map_buffer()
    }

    pub fn unmap_buffer(&self) {
        self.backend_unmap_buffer();
    }

    pub fn mapped_ptr(&self) -> *mut u8 {
        self.backend_mapped_ptr()
    }
}

pub struct BufferCopy {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64,
}
