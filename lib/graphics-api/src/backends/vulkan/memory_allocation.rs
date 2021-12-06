use ash::vk::{self};

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

pub(crate) struct VulkanMemoryPagesAllocation {
    allocated_pages: Vec<(vk_mem::Allocation, vk_mem::AllocationInfo)>,
}

impl VulkanMemoryPagesAllocation {
    pub fn for_sparse_buffer(
        device_context: &DeviceContext,
        buffer: &Buffer,
        page_count: u64,
    ) -> Self {
        let memory_requirements = unsafe {
            device_context
                .platform_device_context()
                .device()
                .get_buffer_memory_requirements(buffer.platform_buffer().vk_buffer())
        };

        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            flags: vk_mem::AllocationCreateFlags::NONE,
            required_flags: vk::MemoryPropertyFlags::empty(),
            preferred_flags: vk::MemoryPropertyFlags::empty(),
            memory_type_bits: 0, // Do not exclude any memory types
            pool: None,
            user_data: None,
        };

        let allocated_pages = device_context
            .platform_device_context()
            .allocator()
            .allocate_memory_pages(
                &memory_requirements,
                &allocation_create_info,
                page_count as usize,
            )
            .unwrap();

        Self { allocated_pages }
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) {
        let mut allocations = Vec::with_capacity(self.allocated_pages.len());
        for allocation in &self.allocated_pages {
            allocations.push(allocation.0);
        }
        device_context
            .platform_device_context()
            .allocator()
            .free_memory_pages(&allocations);

        self.allocated_pages.clear();
    }

    pub fn binding_info(
        &self,
        sparse_binding_info: &mut SparseBindingInfo<'_>,
    ) -> ash::vk::SparseBufferMemoryBindInfo {
        for allocation in &self.allocated_pages {
            let mut builder = ash::vk::SparseMemoryBind::builder()
                .resource_offset(sparse_binding_info.buffer_offset)
                .size(allocation.1.get_size() as u64);

            if sparse_binding_info.bind {
                builder = builder
                    .memory(allocation.1.get_device_memory())
                    .memory_offset(allocation.1.get_offset() as u64);
            }

            sparse_binding_info.sparse_bindings.push(*builder);
            sparse_binding_info.buffer_offset += allocation.1.get_size() as u64;
        }

        *ash::vk::SparseBufferMemoryBindInfo::builder()
            .buffer(sparse_binding_info.buffer.platform_buffer().vk_buffer())
            .binds(&sparse_binding_info.sparse_bindings)
    }
}

pub(crate) struct SparseBindingInfo<'a> {
    pub(crate) sparse_bindings: Vec<ash::vk::SparseMemoryBind>,
    pub(crate) buffer_offset: u64,
    pub(crate) buffer: &'a Buffer,
    pub(crate) bind: bool,
}
