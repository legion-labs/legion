use lgn_tracing::trace;

use crate::{Buffer, BufferCopy, BufferDef, BufferMappingInfo, DeviceContext, ResourceUsage};

#[derive(Debug)]
pub(crate) struct VulkanBuffer {
    vk_allocation: vk_mem::Allocation,
    vk_allocation_info: vk_mem::AllocationInfo,
    vk_buffer: ash::vk::Buffer,
}

impl VulkanBuffer {
    pub fn new(device_context: &DeviceContext, buffer_def: BufferDef) -> Self {
        trace!("creating VulkanBuffer");

        buffer_def.verify();
        let allocation_size = if buffer_def
            .usage_flags
            .intersects(ResourceUsage::AS_CONST_BUFFER)
        {
            lgn_utils::memory::round_size_up_to_alignment_u64(
                buffer_def.size,
                device_context.limits().min_uniform_buffer_offset_alignment,
            )
        } else {
            buffer_def.size
        };

        let mut usage_flags =
            super::internal::resource_type_buffer_usage_flags(buffer_def.usage_flags);

        if buffer_def
            .usage_flags
            .intersects(ResourceUsage::AS_TRANSFERABLE)
        {
            usage_flags |= ash::vk::BufferUsageFlags::TRANSFER_DST;
        }

        let creation_flags =
            super::internal::resource_type_buffer_creation_flags(buffer_def.create_flags);

        assert_ne!(allocation_size, 0);

        let buffer_info = ash::vk::BufferCreateInfo::builder()
            .flags(creation_flags)
            .size(allocation_size)
            .usage(usage_flags)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let mut alloc_flags = vk_mem::AllocationCreateFlags::NONE;
        if buffer_def.always_mapped {
            alloc_flags |= vk_mem::AllocationCreateFlags::MAPPED;
        }

        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: buffer_def.memory_usage.into(),
            flags: alloc_flags,
            required_flags: ash::vk::MemoryPropertyFlags::empty(),
            preferred_flags: ash::vk::MemoryPropertyFlags::empty(),
            memory_type_bits: 0, // Do not exclude any memory types
            pool: None,
            user_data: None,
        };

        let (vk_buffer, vk_allocation, vk_allocation_info) = device_context
            .vk_allocator()
            .create_buffer(&buffer_info, &allocation_create_info)
            .unwrap();

        trace!(
            "Buffer {:?} created with size {}",
            vk_buffer,
            buffer_info.size,
        );

        Self {
            vk_allocation,
            vk_allocation_info,
            vk_buffer,
        }
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext, buffer_def: &BufferDef) {
        trace!("destroying VulkanBuffer");

        trace!(
            "Buffer {:?} destroying with size {}",
            self.vk_buffer,
            buffer_def.size,
        );

        unsafe {
            device_context
                .vk_device()
                .destroy_buffer(self.vk_buffer, None);
        };

        trace!("destroyed VulkanBuffer");
    }
}

impl Buffer {
    pub(crate) fn vk_buffer(&self) -> ash::vk::Buffer {
        self.inner.backend_buffer.vk_buffer
    }

    pub(crate) fn backend_map_buffer(&self) -> BufferMappingInfo<'_> {
        let ptr = self
            .inner
            .device_context
            .vk_allocator()
            .map_memory(&self.inner.backend_buffer.vk_allocation)
            .unwrap();

        BufferMappingInfo {
            _buffer: self,
            data_ptr: ptr,
        }
    }

    pub(crate) fn backend_unmap_buffer(&self) {
        self.inner
            .device_context
            .vk_allocator()
            .unmap_memory(&self.inner.backend_buffer.vk_allocation);
    }

    pub(crate) fn backend_mapped_ptr(&self) -> *mut u8 {
        self.inner
            .backend_buffer
            .vk_allocation_info
            .get_mapped_data()
    }
}

impl From<&BufferCopy> for ash::vk::BufferCopy {
    fn from(src: &BufferCopy) -> Self {
        Self::builder()
            .src_offset(src.src_offset)
            .dst_offset(src.dst_offset)
            .size(src.size)
            .build()
    }
}
