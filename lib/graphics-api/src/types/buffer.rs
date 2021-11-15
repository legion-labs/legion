#[cfg(feature = "vulkan")]
use crate::backends::vulkan::{VulkanBuffer, VulkanDeviceContext};
use crate::{BufferView, GfxResult};

use super::{
    deferred_drop::Drc, BufferViewDef, DeviceContext, MemoryUsage, QueueType, ResourceUsage,
};

#[derive(Clone, Debug, Default)]
pub struct BufferElementData {
    // For storage buffers
    pub element_begin_index: u64,
    pub element_count: u64,
    pub element_stride: u64,
}

/// Used to create a `Buffer`
#[derive(Clone, Copy, Debug)]
pub struct BufferDef {
    pub size: u64,
    pub memory_usage: MemoryUsage,
    pub queue_type: QueueType,
    pub always_mapped: bool,
    pub usage_flags: ResourceUsage,
}

impl Default for BufferDef {
    fn default() -> Self {
        Self {
            size: 0,
            memory_usage: MemoryUsage::Unknown,
            queue_type: QueueType::Graphics,
            always_mapped: false,
            usage_flags: ResourceUsage::empty(),
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
            memory_usage: MemoryUsage::CpuToGpu,
            queue_type: QueueType::Graphics,
            always_mapped: false,
            usage_flags,
        }
    }

    pub fn for_staging_buffer_data<T: Copy>(data: &[T], usage_flags: ResourceUsage) -> Self {
        Self::for_staging_buffer(legion_utils::memory::slice_size_in_bytes(data), usage_flags)
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

pub struct BufferInner {
    buffer_def: BufferDef,
    device_context: DeviceContext,

    #[cfg(feature = "vulkan")]
    pub(super) platform_buffer: VulkanBuffer,
}

impl Drop for BufferInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_buffer.destroy(
            &self.device_context.inner.platform_device_context,
            &self.buffer_def,
        );
    }
}

#[derive(Clone)]
pub struct Buffer {
    inner: Drc<BufferInner>,
}

impl Buffer {
    pub fn new(device_context: &DeviceContext, buffer_def: &BufferDef) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_buffer =
            VulkanBuffer::new(&device_context.inner.platform_device_context, buffer_def).map_err(
                |e| {
                    log::error!("Error creating buffer {:?}", e);
                    ash::vk::Result::ERROR_UNKNOWN
                },
            )?;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(BufferInner {
                device_context: device_context.clone(),
                buffer_def: *buffer_def,
                #[cfg(any(feature = "vulkan"))]
                platform_buffer,
            }),
        })
    }

    pub fn definition(&self) -> &BufferDef {
        &self.inner.buffer_def
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_buffer(&self) -> &VulkanBuffer {
        &self.inner.platform_buffer
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_device_context(&self) -> &VulkanDeviceContext {
        &self.inner.device_context.inner.platform_device_context
    }

    pub fn map_buffer(&self) -> GfxResult<BufferMappingInfo> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        {
            let ptr = self
                .inner
                .platform_buffer
                .map_buffer(self.platform_device_context())?;

            Ok(BufferMappingInfo {
                buffer: self.clone(),
                data_ptr: ptr,
            })
        }
    }

    pub fn unmap_buffer(&self) {
        #[cfg(any(feature = "vulkan"))]
        self.inner
            .platform_buffer
            .unmap_buffer(self.platform_device_context());
    }

    pub fn copy_to_host_visible_buffer<T: Copy>(&self, data: &[T]) -> GfxResult<()> {
        // Cannot check size of data == buffer because buffer size might be rounded up
        self.copy_to_host_visible_buffer_with_offset(data, 0)
    }

    pub fn copy_to_host_visible_buffer_with_offset<T: Copy>(
        &self,
        data: &[T],
        buffer_byte_offset: u64,
    ) -> GfxResult<()> {
        let data_size_in_bytes = legion_utils::memory::slice_size_in_bytes(data) as u64;
        assert!(buffer_byte_offset + data_size_in_bytes <= self.inner.buffer_def.size);

        let src = data.as_ptr().cast::<u8>();

        let required_alignment = std::mem::align_of::<T>();

        let mapping_info = self.map_buffer()?;

        #[allow(unsafe_code)]
        unsafe {
            let dst = mapping_info.data_ptr().add(buffer_byte_offset as usize);
            assert_eq!(((dst as usize) % required_alignment), 0);
            std::ptr::copy_nonoverlapping(src, dst, data_size_in_bytes as usize);
        }

        Ok(())
    }

    pub fn create_view(&self, view_def: &BufferViewDef) -> GfxResult<BufferView> {
        BufferView::from_buffer(self, view_def)
    }
}

pub struct BufferMappingInfo {
    buffer: Buffer,
    data_ptr: *mut u8,
}

impl BufferMappingInfo {
    fn data_ptr(&self) -> *mut u8 {
        self.data_ptr
    }
}

impl Drop for BufferMappingInfo {
    fn drop(&mut self) {
        self.buffer.unmap_buffer();
    }
}
