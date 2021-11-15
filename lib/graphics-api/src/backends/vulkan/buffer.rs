use ash::vk;

use super::VulkanDeviceContext;
use crate::{BufferDef, GfxResult, MemoryUsage, ResourceUsage};

#[derive(Debug)]
pub(crate) struct VulkanBuffer {
    allocation_info: vk_mem::AllocationInfo,
    allocation: vk_mem::Allocation,
    buffer: vk::Buffer,
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
            allocation_info,
            allocation,
            buffer,
        })
    }

    pub fn destroy(&self, device_context: &VulkanDeviceContext, buffer_def: &BufferDef) {
        log::trace!("destroying BufferVulkanInner");

        log::trace!(
            "Buffer {:?} destroying with size {} (always mapped: {:?})",
            self.buffer,
            buffer_def.size,
            buffer_def.always_mapped
        );

        device_context
            .allocator()
            .destroy_buffer(self.buffer, &self.allocation);

        log::trace!("destroyed BufferVulkanInner");
    }

    pub fn map_buffer(&self, device_context: &VulkanDeviceContext) -> GfxResult<*mut u8> {
        let ptr = device_context
            .allocator()
            .map_memory(&self.allocation)
            .map_err(|e| {
                log::error!("Error mapping buffer {:?}", e);
                vk::Result::ERROR_UNKNOWN
            })?;
        Ok(ptr)
    }

    pub fn unmap_buffer(&self, device_context: &VulkanDeviceContext) {
        device_context.allocator().unmap_memory(&self.allocation);
    }

    pub fn vk_buffer(&self) -> vk::Buffer {
        self.buffer
    }
}
