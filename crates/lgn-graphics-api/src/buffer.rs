use crate::{
    backends::BackendBuffer, deferred_drop::Drc, BufferView, BufferViewDef, DeviceContext,
    ResourceCreation, ResourceUsage,
};

/// Used to create a `Buffer`
#[derive(Clone, Copy, Debug)]
pub struct BufferDef {
    pub size: u64,
    pub usage_flags: ResourceUsage,
    pub creation_flags: ResourceCreation,
}

impl Default for BufferDef {
    fn default() -> Self {
        Self {
            size: 0,
            usage_flags: ResourceUsage::empty(),
            creation_flags: ResourceCreation::empty(),
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
            creation_flags: ResourceCreation::empty(),
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

    pub fn for_staging_uniform_buffer(size: usize) -> Self {
        Self::for_staging_buffer(size, ResourceUsage::AS_CONST_BUFFER)
    }

    pub fn for_staging_uniform_buffer_data<T: Copy>(data: &[T]) -> Self {
        Self::for_staging_buffer_data(data, ResourceUsage::AS_CONST_BUFFER)
    }
}

pub(crate) struct BufferInner {
    pub(crate) buffer_def: BufferDef,
    pub(crate) device_context: DeviceContext,
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

impl Buffer {
    pub fn new(device_context: &DeviceContext, buffer_def: &BufferDef) -> Self {
        let platform_buffer = BackendBuffer::new(device_context, buffer_def);

        Self {
            inner: device_context.deferred_dropper().new_drc(BufferInner {
                device_context: device_context.clone(),
                buffer_def: *buffer_def,
                backend_buffer: platform_buffer,
            }),
        }
    }

    pub fn definition(&self) -> &BufferDef {
        &self.inner.buffer_def
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn required_alignment(&self) -> u64 {
        self.backend_required_alignment()
    }

    pub fn create_view(&self, view_def: &BufferViewDef) -> BufferView {
        BufferView::from_buffer(self, view_def)
    }
}

pub struct BufferCopy {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64,
}
