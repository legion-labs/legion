use lgn_tracing::trace;

use crate::{Buffer, BufferCopy, BufferDef, DeviceContext, ResourceUsage};

#[derive(Debug)]
pub(crate) struct VulkanBuffer {
    vk_buffer: ash::vk::Buffer,
    vk_mem_requirements: ash::vk::MemoryRequirements,
}

impl VulkanBuffer {
    pub fn new(device_context: &DeviceContext, buffer_def: &BufferDef) -> Self {
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
            super::internal::resource_type_buffer_creation_flags(buffer_def.creation_flags);

        assert_ne!(allocation_size, 0);

        let buffer_info = ash::vk::BufferCreateInfo::builder()
            .flags(creation_flags)
            .size(allocation_size)
            .usage(usage_flags)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let vk_buffer = unsafe {
            device_context
                .vk_device()
                .create_buffer(&buffer_info, None)
                .unwrap()
        };

        let vk_mem_requirements = unsafe {
            device_context
                .vk_device()
                .get_buffer_memory_requirements(vk_buffer)
        };

        trace!(
            "Buffer {:?} created with size {}",
            vk_buffer,
            buffer_info.size,
        );

        Self {
            vk_buffer,
            vk_mem_requirements,
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

    pub(crate) fn backend_required_alignment(&self) -> u64 {
        self.inner.backend_buffer.vk_mem_requirements.alignment
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
