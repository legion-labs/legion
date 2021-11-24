use ash::vk;

use crate::{Buffer, DeviceContext, MemoryAllocationDef};

pub(crate) struct VulkanMemoryAllocation {
    allocation_info: vk_mem::AllocationInfo,
    allocation: vk_mem::Allocation,
}

impl VulkanMemoryAllocation {
    pub fn from_buffer(
        device_context: &DeviceContext,
        buffer: &Buffer,
        alloc_def: &MemoryAllocationDef,
    ) -> Self {
        let mut flags = vk_mem::AllocationCreateFlags::NONE;
        if alloc_def.always_mapped {
            flags |= vk_mem::AllocationCreateFlags::MAPPED;
        }

        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: alloc_def.memory_usage.into(),
            flags,
            required_flags: vk::MemoryPropertyFlags::empty(),
            preferred_flags: vk::MemoryPropertyFlags::empty(),
            memory_type_bits: 0, // Do not exclude any memory types
            pool: None,
            user_data: None,
        };

        //TODO: Better way of handling allocator errors
        let (allocation, allocation_info) = device_context
            .platform_device_context()
            .allocator()
            .allocate_memory_for_buffer(
                buffer.platform_buffer().vk_buffer(),
                &allocation_create_info,
            )
            .map_err(|e| {
                log::error!("Error creating buffer {:?}", e);
                vk::Result::ERROR_UNKNOWN
            })?;

        Self {
            allocation_info,
            allocation,
        }
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
}
