use super::buffer_allocation::BufferSubAllocation;
#[cfg(feature = "vulkan")]
use crate::backends::vulkan::{VulkanMemoryAllocation, VulkanMemoryPagesAllocation};
use crate::{deferred_drop::Drc, Buffer, DeviceContext, MemoryUsage};

pub struct BufferMappingInfo {
    pub allocation: MemoryAllocation,
    pub data_ptr: *mut u8,
}

impl BufferMappingInfo {
    pub fn data_ptr(&self) -> *mut u8 {
        self.data_ptr
    }
}

impl Drop for BufferMappingInfo {
    fn drop(&mut self) {
        self.allocation
            .unmap_buffer(self.allocation.device_context());
    }
}

pub struct MemoryAllocationDef {
    pub memory_usage: MemoryUsage,
    pub always_mapped: bool,
}

impl Default for MemoryAllocationDef {
    fn default() -> Self {
        Self {
            memory_usage: MemoryUsage::Unknown,
            always_mapped: false,
        }
    }
}

pub(crate) struct MemoryAllocationInner {
    device_context: DeviceContext,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_allocation: VulkanMemoryAllocation,
}

impl Drop for MemoryAllocationInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_allocation.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct MemoryAllocation {
    pub(crate) inner: Drc<MemoryAllocationInner>,
}

impl MemoryAllocation {
    pub fn from_buffer(
        device_context: &DeviceContext,
        buffer: &Buffer,
        alloc_def: &MemoryAllocationDef,
    ) -> Self {
        #[cfg(feature = "vulkan")]
        let platform_allocation =
            VulkanMemoryAllocation::from_buffer(device_context, buffer, alloc_def);

        Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(MemoryAllocationInner {
                    device_context: device_context.clone(),
                    #[cfg(any(feature = "vulkan"))]
                    platform_allocation,
                }),
        }
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
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

        #[cfg(any(feature = "vulkan"))]
        assert!(buffer_byte_offset + data_size_in_bytes <= self.size() as u64);

        let src = data.as_ptr().cast::<u8>();

        let required_alignment = std::mem::align_of::<T>();

        let mapping_info = self.map_buffer(self.device_context());

        #[allow(unsafe_code)]
        unsafe {
            let dst = mapping_info.data_ptr().add(buffer_byte_offset as usize);
            assert_eq!(((dst as usize) % required_alignment), 0);
            std::ptr::copy_nonoverlapping(src, dst, data_size_in_bytes as usize);
        }
    }

    pub fn map_buffer(&self, device_context: &DeviceContext) -> BufferMappingInfo {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(feature = "vulkan")]
        self.map_buffer_platform(device_context)
    }

    pub fn unmap_buffer(&self, device_context: &DeviceContext) {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(feature = "vulkan")]
        self.unmap_buffer_platform(device_context);
    }

    pub fn mapped_ptr(&self) -> *mut u8 {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(feature = "vulkan")]
        self.mapped_ptr_platform()
    }

    pub fn size(&self) -> usize {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(feature = "vulkan")]
        self.size_platform()
    }
}

pub struct MemoryMappingInfo {
    allocation: MemoryAllocation,
    data_ptr: *mut u8,
}

impl MemoryMappingInfo {
    pub fn data_ptr(&self) -> *mut u8 {
        self.data_ptr
    }
}

impl Drop for MemoryMappingInfo {
    fn drop(&mut self) {
        self.allocation
            .unmap_buffer(self.allocation.device_context());
    }
}

pub(crate) struct MemoryPagesAllocationInner {
    device_context: DeviceContext,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_allocation: VulkanMemoryPagesAllocation,
}

impl Drop for MemoryPagesAllocationInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_allocation.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct MemoryPagesAllocation {
    pub(crate) inner: Drc<MemoryPagesAllocationInner>,
}

impl MemoryPagesAllocation {
    pub fn for_sparse_buffer(
        device_context: &DeviceContext,
        buffer: &Buffer,
        page_count: u64,
    ) -> Self {
        #[cfg(feature = "vulkan")]
        let platform_allocation =
            VulkanMemoryPagesAllocation::for_sparse_buffer(device_context, buffer, page_count);

        Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(MemoryPagesAllocationInner {
                    device_context: device_context.clone(),
                    #[cfg(any(feature = "vulkan"))]
                    platform_allocation,
                }),
        }
    }

    pub fn empty_allocation(device_context: &DeviceContext) -> Self {
        #[cfg(feature = "vulkan")]
        let platform_allocation = VulkanMemoryPagesAllocation::empty_allocation();
        Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(MemoryPagesAllocationInner {
                    device_context: device_context.clone(),
                    #[cfg(any(feature = "vulkan"))]
                    platform_allocation,
                }),
        }
    }
}

pub type BufferAllocation = BufferSubAllocation<MemoryAllocation>;
pub type PagedBufferAllocation = BufferSubAllocation<MemoryPagesAllocation>;
