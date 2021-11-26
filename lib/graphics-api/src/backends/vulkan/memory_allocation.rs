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

        let (allocation, allocation_info) = device_context
            .platform_device_context()
            .allocator()
            .allocate_memory_for_buffer(
                buffer.platform_buffer().vk_buffer(),
                &allocation_create_info,
            )
            .unwrap();

        device_context
            .platform_device_context()
            .allocator()
            .bind_buffer_memory(buffer.platform_buffer().vk_buffer(), &allocation)
            .unwrap();

        Self {
            allocation_info,
            allocation,
        }
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        device_context
            .platform_device_context()
            .allocator()
            .free_memory(&self.allocation);
    }

    pub fn map_buffer(&self, device_context: &DeviceContext) -> *mut u8 {
        device_context
            .platform_device_context()
            .allocator()
            .map_memory(&self.allocation)
            .unwrap()
    }

    pub fn unmap_buffer(&self, device_context: &DeviceContext) {
        device_context
            .platform_device_context()
            .allocator()
            .unmap_memory(&self.allocation);
    }

    pub fn mapped_ptr(&self) -> *mut u8 {
        self.allocation_info.get_mapped_data()
    }

    pub fn size(&self) -> usize {
        self.allocation_info.get_size()
    }
}
