use crate::{Buffer, BufferDef, DeviceContext, GfxResult, MemoryUsage, ResourceUsage};

#[derive(Debug)]
pub(crate) struct VulkanBuffer {
    _allocation_info: vk_mem::AllocationInfo,
    allocation: vk_mem::Allocation,
    vk_buffer: ash::vk::Buffer,
}

impl VulkanBuffer {
    pub fn new(device_context: &DeviceContext, buffer_def: &BufferDef) -> GfxResult<Self> {
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

        if buffer_def.memory_usage == MemoryUsage::GpuOnly
            || buffer_def.memory_usage == MemoryUsage::CpuToGpu
        {
            usage_flags |= ash::vk::BufferUsageFlags::TRANSFER_DST;
        }

        let mut flags = vk_mem::AllocationCreateFlags::NONE;
        if buffer_def.always_mapped {
            flags |= vk_mem::AllocationCreateFlags::MAPPED;
        }

        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: buffer_def.memory_usage.into(),
            flags,
            required_flags: ash::vk::MemoryPropertyFlags::empty(),
            preferred_flags: ash::vk::MemoryPropertyFlags::empty(),
            memory_type_bits: 0, // Do not exclude any memory types
            pool: None,
            user_data: None,
        };

        assert_ne!(allocation_size, 0);

        let buffer_info = ash::vk::BufferCreateInfo::builder()
            .size(allocation_size)
            .usage(usage_flags)
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        //TODO: Better way of handling allocator errors
        let (buffer, allocation, allocation_info) = device_context
            .vk_allocator()
            .create_buffer(&buffer_info, &allocation_create_info)
            .map_err(|e| {
                log::error!("Error creating buffer {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;

        log::trace!(
            "Buffer {:?} crated with size {} (always mapped: {:?})",
            buffer,
            buffer_info.size,
            buffer_def.always_mapped
        );

        Ok(Self {
            _allocation_info: allocation_info,
            allocation,
            vk_buffer: buffer,
        })
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext, buffer_def: &BufferDef) {
        log::trace!("destroying VulkanBuffer");

        log::trace!(
            "Buffer {:?} destroying with size {} (always mapped: {:?})",
            self.vk_buffer,
            buffer_def.size,
            buffer_def.always_mapped
        );

        device_context
            .vk_allocator()
            .destroy_buffer(self.vk_buffer, &self.allocation);

        log::trace!("destroyed VulkanBuffer");
    }
}

impl Buffer {
    pub(crate) fn vk_buffer(&self) -> ash::vk::Buffer {
        self.inner.platform_buffer.vk_buffer
    }

    pub(crate) fn map_buffer_platform(&self) -> GfxResult<*mut u8> {
        let ptr = self
            .inner
            .device_context
            .vk_allocator()
            .map_memory(&self.inner.platform_buffer.allocation)
            .map_err(|e| {
                log::error!("Error mapping buffer {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;
        Ok(ptr)
    }

    pub(crate) fn unmap_buffer_platform(&self) {
        self.inner
            .device_context
            .vk_allocator()
            .unmap_memory(&self.inner.platform_buffer.allocation);
    }
}
