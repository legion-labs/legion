use ash::vk;

use super::{VulkanApi, VulkanBufferView, VulkanDeviceContext};
use crate::backends::deferred_drop::Drc;
use crate::{
    Buffer, BufferDef, BufferMappingInfo, BufferViewDef, GfxResult, MemoryUsage, ResourceUsage,
};

#[derive(Debug)]
struct VulkanBufferInner {
    buffer_def: BufferDef,
    device_context: VulkanDeviceContext,
    allocation_info: vk_mem::AllocationInfo,
    allocation: vk_mem::Allocation,
    buffer: vk::Buffer,
}

#[derive(Clone, Debug)]
pub struct VulkanBuffer {
    inner: Drc<VulkanBufferInner>,
}

impl VulkanBuffer {
    pub fn new(device_context: &VulkanDeviceContext, buffer_def: &BufferDef) -> GfxResult<Self> {
        buffer_def.verify();
        let mut allocation_size = buffer_def.size;

        if buffer_def
            .usage_flags
            .intersects(ResourceUsage::AS_CONST_BUFFER)
        {
            allocation_size = legion_utils::memory::round_size_up_to_alignment_u64(
                buffer_def.size,
                device_context.limits().min_uniform_buffer_offset_alignment,
            );
        }

        let mut usage_flags =
            super::internal::resource_type_buffer_usage_flags(buffer_def.usage_flags);

        if buffer_def.memory_usage == MemoryUsage::GpuOnly
            || buffer_def.memory_usage == MemoryUsage::CpuToGpu
        {
            usage_flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        let mut flags = vk_mem::AllocationCreateFlags::NONE;
        if buffer_def.always_mapped {
            flags |= vk_mem::AllocationCreateFlags::MAPPED;
        }

        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: buffer_def.memory_usage.into(),
            flags,
            required_flags: vk::MemoryPropertyFlags::empty(),
            preferred_flags: vk::MemoryPropertyFlags::empty(),
            memory_type_bits: 0, // Do not exclude any memory types
            pool: None,
            user_data: None,
        };

        assert_ne!(allocation_size, 0);

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(allocation_size)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        //TODO: Better way of handling allocator errors
        let (buffer, allocation, allocation_info) = device_context
            .allocator()
            .create_buffer(&buffer_info, &allocation_create_info)
            .map_err(|e| {
                log::error!("Error creating buffer {:?}", e);
                vk::Result::ERROR_UNKNOWN
            })?;

        log::trace!(
            "Buffer {:?} crated with size {} (always mapped: {:?})",
            buffer,
            buffer_info.size,
            buffer_def.always_mapped
        );

        Ok(Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(VulkanBufferInner {
                    device_context: device_context.clone(),
                    allocation_info,
                    buffer_def: *buffer_def,
                    allocation,
                    buffer,
                }),
        })
    }

    pub fn device_context(&self) -> &VulkanDeviceContext {
        &self.inner.device_context
    }

    pub fn vk_buffer(&self) -> vk::Buffer {
        self.inner.buffer
    }
}

impl Drop for VulkanBufferInner {
    fn drop(&mut self) {
        log::trace!("destroying BufferVulkanInner");

        log::trace!(
            "Buffer {:?} destroying with size {} (always mapped: {:?})",
            self.buffer,
            self.buffer_def.size,
            self.buffer_def.always_mapped
        );

        self.device_context
            .allocator()
            .destroy_buffer(self.buffer, &self.allocation);

        log::trace!("destroyed BufferVulkanInner");
    }
}

pub struct VulkanBufferMappingInfo {
    buffer: VulkanBuffer,
    data_ptr: *mut u8,
}

impl Drop for VulkanBufferMappingInfo {
    fn drop(&mut self) {
        self.buffer
            .device_context()
            .allocator()
            .unmap_memory(&self.buffer.inner.allocation);
    }
}

impl BufferMappingInfo<VulkanApi> for VulkanBufferMappingInfo {
    fn data_ptr(&self) -> *mut u8 {
        self.data_ptr
    }
}

impl Buffer<VulkanApi> for VulkanBuffer {
    fn definition(&self) -> &BufferDef {
        &self.inner.buffer_def
    }

    fn map_buffer(&self) -> GfxResult<VulkanBufferMappingInfo> {
        let ptr = self
            .inner
            .device_context
            .allocator()
            .map_memory(&self.inner.allocation)?;
        Ok(VulkanBufferMappingInfo {
            buffer: self.clone(),
            data_ptr: ptr,
        })
    }

    fn copy_to_host_visible_buffer<T: Copy>(&self, data: &[T]) -> GfxResult<()> {
        // Cannot check size of data == buffer because buffer size might be rounded up
        self.copy_to_host_visible_buffer_with_offset(data, 0)
    }

    fn copy_to_host_visible_buffer_with_offset<T: Copy>(
        &self,
        data: &[T],
        buffer_byte_offset: u64,
    ) -> GfxResult<()> {
        let data_size_in_bytes = legion_utils::memory::slice_size_in_bytes(data) as u64;
        assert!(buffer_byte_offset + data_size_in_bytes <= self.inner.buffer_def.size);

        let src = data.as_ptr().cast::<u8>();

        let required_alignment = std::mem::align_of::<T>();

        let mapping_info = self.map_buffer()?;
        unsafe {
            let dst = mapping_info.data_ptr().add(buffer_byte_offset as usize);
            assert_eq!(((dst as usize) % required_alignment), 0);
            std::ptr::copy_nonoverlapping(src, dst, data_size_in_bytes as usize);
        }

        Ok(())
    }

    fn create_view(&self, view_def: &BufferViewDef) -> GfxResult<VulkanBufferView> {
        VulkanBufferView::from_buffer(self, view_def)
    }
}
