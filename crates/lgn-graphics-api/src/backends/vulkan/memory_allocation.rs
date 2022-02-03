use ash::vk::{self};

use crate::{
    Buffer, BufferMappingInfo, DeviceContext, MemoryAllocation, MemoryAllocationDef,
    MemoryPagesAllocation,
};

pub(crate) struct VulkanMemoryAllocation {
    vk_allocation_info: vk_mem::AllocationInfo,
    vk_allocation: vk_mem::Allocation,
}

impl VulkanMemoryAllocation {
    pub(crate) fn from_buffer(
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

        let (vk_allocation, vk_allocation_info) = device_context
            .vk_allocator()
            .allocate_memory_for_buffer(buffer.vk_buffer(), &allocation_create_info)
            .unwrap();

        device_context
            .vk_allocator()
            .bind_buffer_memory(buffer.vk_buffer(), &vk_allocation)
            .unwrap();

        Self {
            vk_allocation_info,
            vk_allocation,
        }
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        device_context
            .vk_allocator()
            .free_memory(&self.vk_allocation);
    }
}

impl MemoryAllocation {
    pub(crate) fn backend_map_buffer(&self, device_context: &DeviceContext) -> BufferMappingInfo {
        let ptr = device_context
            .vk_allocator()
            .map_memory(self.vk_allocation())
            .unwrap();

        BufferMappingInfo {
            allocation: self.clone(),
            data_ptr: ptr,
        }
    }

    pub(crate) fn backend_unmap_buffer(&self, device_context: &DeviceContext) {
        device_context
            .vk_allocator()
            .unmap_memory(self.vk_allocation());
    }

    pub(crate) fn backend_mapped_ptr(&self) -> *mut u8 {
        self.vk_allocation_info().get_mapped_data()
    }

    pub(crate) fn backend_size(&self) -> usize {
        self.vk_allocation_info().get_size()
    }

    pub(crate) fn vk_allocation(&self) -> &vk_mem::Allocation {
        &self.inner.backend_allocation.vk_allocation
    }

    pub(crate) fn vk_allocation_info(&self) -> &vk_mem::AllocationInfo {
        &self.inner.backend_allocation.vk_allocation_info
    }
}

pub(crate) struct VulkanMemoryPagesAllocation {
    vk_allocated_pages: Vec<(vk_mem::Allocation, vk_mem::AllocationInfo)>,
}

impl VulkanMemoryPagesAllocation {
    pub fn for_sparse_buffer(
        device_context: &DeviceContext,
        buffer: &Buffer,
        page_count: u64,
    ) -> Self {
        let mut memory_requirements = unsafe {
            device_context
                .vk_device()
                .get_buffer_memory_requirements(buffer.vk_buffer())
        };
        memory_requirements.size = memory_requirements.alignment;

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
            .vk_allocator()
            .allocate_memory_pages(
                &memory_requirements,
                &allocation_create_info,
                page_count as usize,
            )
            .unwrap();

        Self {
            vk_allocated_pages: allocated_pages,
        }
    }

    pub fn empty_allocation() -> Self {
        Self {
            vk_allocated_pages: Vec::new(),
        }
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) {
        let mut allocations = Vec::with_capacity(self.vk_allocated_pages.len());
        for allocation in &self.vk_allocated_pages {
            allocations.push(allocation.0);
        }
        device_context
            .vk_allocator()
            .free_memory_pages(&allocations);

        self.vk_allocated_pages.clear();
    }
}

impl MemoryPagesAllocation {
    pub fn vk_allocated_pages(&self) -> &Vec<(vk_mem::Allocation, vk_mem::AllocationInfo)> {
        &self.inner.backend_allocation.vk_allocated_pages
    }

    pub fn binding_info(
        &self,
        sparse_binding_info: &mut SparseBindingInfo<'_>,
    ) -> ash::vk::SparseBufferMemoryBindInfo {
        for allocation in self.vk_allocated_pages() {
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
            .buffer(sparse_binding_info.buffer.vk_buffer())
            .binds(&sparse_binding_info.sparse_bindings)
    }
}

pub struct SparseBindingInfo<'a> {
    pub(crate) sparse_bindings: Vec<ash::vk::SparseMemoryBind>,
    pub(crate) buffer_offset: u64,
    pub(crate) buffer: &'a Buffer,
    pub(crate) bind: bool,
}
