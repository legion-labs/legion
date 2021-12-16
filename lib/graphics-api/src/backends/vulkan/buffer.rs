use crate::{Buffer, BufferDef, DeviceContext, ResourceUsage};

#[derive(Debug)]
pub(crate) struct VulkanBuffer {
    vk_buffer: ash::vk::Buffer,
    vk_mem_requirements: ash::vk::MemoryRequirements,
}

impl VulkanBuffer {
    pub fn new(device_context: &DeviceContext, buffer_def: &BufferDef) -> Self {
        buffer_def.verify();
        let mut allocation_size = buffer_def.size;

        if buffer_def
            .usage_flags
            .intersects(ResourceUsage::AS_CONST_BUFFER)
        {
            allocation_size = lgn_utils::memory::round_size_up_to_alignment_u64(
                buffer_def.size,
                device_context.limits().min_uniform_buffer_offset_alignment,
            );
        }

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

        log::trace!(
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
        log::trace!("destroying VulkanBuffer");

        log::trace!(
            "Buffer {:?} destroying with size {}",
            self.vk_buffer,
            buffer_def.size,
        );

        unsafe {
            device_context
                .vk_device()
                .destroy_buffer(self.vk_buffer, None);
        };

        log::trace!("destroyed VulkanBuffer");
    }
}

impl Buffer {
    pub(crate) fn vk_buffer(&self) -> ash::vk::Buffer {
        self.inner.platform_buffer.vk_buffer
    }

    pub(crate) fn required_alignment_platform(&self) -> u64 {
        self.inner.platform_buffer.vk_mem_requirements.alignment
    }
}
