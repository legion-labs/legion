use ash::vk::{self, SparseMemoryBindFlags};

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
    sparse_bindings: Vec<ash::vk::SparseMemoryBind>,
    sparse_unbindings: Vec<ash::vk::SparseMemoryBind>,
    buffer: Buffer,
}

impl VulkanMemoryPagesAllocation {
    pub fn for_sparse_buffer(
        device_context: &DeviceContext,
        buffer: &Buffer,
        buffer_offset: u64,
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

        let mut sparse_bindings = Vec::with_capacity(allocated_pages.len());
        for allocation in &allocated_pages {
            let sparse_binding_builder = ash::vk::SparseMemoryBind::builder()
                .resource_offset(buffer_offset)
                .size(allocation.1.get_size() as u64)
                .memory(allocation.1.get_device_memory())
                .memory_offset(allocation.1.get_offset() as u64);

            sparse_bindings.push(*sparse_binding_builder);
        }

        let mut sparse_unbindings = Vec::with_capacity(allocated_pages.len());
        for allocation in &allocated_pages {
            sparse_unbindings.push(ash::vk::SparseMemoryBind {
                resource_offset: buffer_offset,
                size: allocation.1.get_size() as u64,
                memory: allocation.1.get_device_memory(),
                memory_offset: allocation.1.get_offset() as u64,
                flags: SparseMemoryBindFlags::empty(),
            });
        }

        Self {
            allocated_pages,
            sparse_bindings,
            sparse_unbindings,
            buffer: buffer.clone(),
        }
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

    pub fn binding_info(&self) -> ash::vk::SparseBufferMemoryBindInfo {
        *ash::vk::SparseBufferMemoryBindInfo::builder()
            .buffer(self.buffer.platform_buffer().vk_buffer())
            .binds(&self.sparse_bindings)
    }

    pub fn unbinding_info(&self) -> ash::vk::SparseBufferMemoryBindInfo {
        *ash::vk::SparseBufferMemoryBindInfo::builder()
            .buffer(self.buffer.platform_buffer().vk_buffer())
            .binds(&self.sparse_unbindings)
    }
}
