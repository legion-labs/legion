use ash::vk;

use super::VulkanDeviceContext;
use crate::{BufferDef, ResourceUsage};

#[derive(Debug)]
pub(crate) struct VulkanBuffer {
    buffer: vk::Buffer,
    buffer_info: vk::BufferCreateInfo,
}

impl VulkanBuffer {
    pub fn new(device_context: &VulkanDeviceContext, buffer_def: &BufferDef) -> Self {
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

        if buffer_def
            .usage_flags
            .intersects(ResourceUsage::AS_TRANSFERABLE)
        {
            usage_flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        assert_ne!(allocation_size, 0);

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(allocation_size)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device_context
                .device()
                .create_buffer(&buffer_info, None)
                .unwrap()
        };

        log::trace!(
            "Buffer {:?} crated with size {} (always mapped: {:?})",
            buffer,
            buffer_info.size,
            buffer_def.always_mapped
        );

        Self {
            buffer,
            buffer_info: *buffer_info,
        }
    }

    pub fn destroy(&self, device_context: &VulkanDeviceContext, buffer_def: &BufferDef) {
        log::trace!("destroying BufferVulkanInner");

        log::trace!(
            "Buffer {:?} destroying with size {}",
            self.buffer,
            buffer_def.size,
        );

        unsafe { device_context.device().destroy_buffer(self.buffer, None) };

        log::trace!("destroyed BufferVulkanInner");
    }

    pub fn vk_buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn vk_buffer_info(&self) -> vk::BufferCreateInfo {
        self.buffer_info
    }
}
