use std::sync::Arc;

use super::{VulkanApi, VulkanBufferView, VulkanDeviceContext};
use crate::{Buffer, BufferDef, GfxResult, MemoryUsage, ResourceUsage};
use ash::vk;
use legion_utils::trust_cell::TrustCell;

#[derive(Copy, Clone, Debug)]
pub struct BufferRaw {
    pub buffer: vk::Buffer,
    pub allocation: vk_mem::Allocation,
}

#[derive(Debug)]
struct VulkanBufferInner {
    buffer_def: BufferDef,
    device_context: VulkanDeviceContext,
    allocation_info: TrustCell<vk_mem::AllocationInfo>,
    buffer_raw: Option<BufferRaw>,
}

#[derive(Clone,Debug)]
pub struct VulkanBuffer {
    inner: Arc<VulkanBufferInner>,    
}

impl VulkanBuffer {
    pub fn device_context(&self) -> &VulkanDeviceContext {
        &self.inner.device_context
    }    

    pub fn vk_buffer(&self) -> vk::Buffer {
        self.inner.buffer_raw.unwrap().buffer
    }

    pub fn vk_uniform_texel_view(&self) -> Option<vk::BufferView> {
        // self.uniform_texel_view
        panic!()
    }

    pub fn vk_storage_texel_view(&self) -> Option<vk::BufferView> {
        // self.storage_texel_view
        panic!()
    }

    // pub fn take_raw(mut self) -> Option<BufferRaw> {
    //     let mut raw = None;
    //     std::mem::swap(&mut raw, &mut self.inner.buffer_raw);
    //     raw
    // }

    pub fn new(device_context: &VulkanDeviceContext, buffer_def: &BufferDef) -> GfxResult<Self> {
        buffer_def.verify();
        let mut allocation_size = buffer_def.size;

        if buffer_def.usage.intersects(ResourceUsage::HAS_CONST_BUFFER_VIEW) {
            allocation_size = legion_utils::memory::round_size_up_to_alignment_u64(
                buffer_def.size,
                device_context.limits().min_uniform_buffer_offset_alignment,
            );
        }

        // if buffer_def
        //     .resource_type
        //     .intersects(ResourceType::UNIFORM_BUFFER_)
        // {
        //     allocation_size = legion_utils::memory::round_size_up_to_alignment_u64(
        //         buffer_def.size,
        //         device_context.limits().min_uniform_buffer_offset_alignment,
        //     );
        // }

        let mut usage_flags = super::internal::resource_type_buffer_usage_flags(
            buffer_def.usage,
            // buffer_def.format != Format::UNDEFINED,
        );

        if buffer_def.memory_usage == MemoryUsage::GpuOnly
            || buffer_def.memory_usage == MemoryUsage::CpuToGpu
        {
            usage_flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        let mut flags = vk_mem::AllocationCreateFlags::NONE;
        if buffer_def.always_mapped {
            flags |= vk_mem::AllocationCreateFlags::MAPPED;
        }

        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: buffer_def.memory_usage.into(),
            flags,
            required_flags: vk::MemoryPropertyFlags::empty(),
            preferred_flags: vk::MemoryPropertyFlags::empty(),
            memory_type_bits: 0, // Do not exclude any memory types
            pool: None,
            user_data: None,
        };

        assert_ne!(allocation_size, 0);

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(allocation_size)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        //TODO: Better way of handling allocator errors
        let (buffer, allocation, allocation_info) = device_context
            .allocator()
            .create_buffer(&buffer_info, &allocation_create_info)
            .map_err(|e| {
                log::error!("Error creating buffer {:?}", e);
                vk::Result::ERROR_UNKNOWN
            })?;

        let buffer_raw = BufferRaw { buffer, allocation };

        log::trace!(
            "Buffer {:?} crated with size {} (always mapped: {:?})",
            buffer_raw.buffer,
            buffer_info.size,
            buffer_def.always_mapped
        );

        // let mut buffer_offset = 0;
        // if buffer_def.resource_type.intersects(ResourceType::BUFFER | ResourceType::BUFFER_READ_WRITE) {
        //     buffer_offset = buffer_def.struct_stride * buffer_def.first_element;
        // }

        // let _uniform_texel_view = if usage_flags
        //     .intersects(vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER)
        // {
        //     let create_info = vk::BufferViewCreateInfo::builder()
        //         .buffer(buffer_raw.buffer)
        //         .format(buffer_def.format.into())
        //         .offset(
        //             buffer_def.elements.element_stride * buffer_def.elements.element_begin_index,
        //         )
        //         .range(
        //             buffer_def.elements.element_stride * buffer_def.elements.element_begin_index,
        //         );

        //     //TODO: Verify we support the format
        //     unsafe {
        //         Some(
        //             device_context
        //                 .device()
        //                 .create_buffer_view(&*create_info, None)?,
        //         )
        //     }
        // } else {
        //     None
        // };

        // let storage_texel_view = if usage_flags
        //     .intersects(vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER)
        // {
        //     let create_info = vk::BufferViewCreateInfo::builder()
        //         .buffer(buffer_raw.buffer)
        //         .format(buffer_def.format.into())
        //         .offset(
        //             buffer_def.elements.element_stride * buffer_def.elements.element_begin_index,
        //         )
        //         .range(
        //             buffer_def.elements.element_stride * buffer_def.elements.element_begin_index,
        //         );

        //     //TODO: Verify we support the format
        //     unsafe {
        //         Some(
        //             device_context
        //                 .device()
        //                 .create_buffer_view(&*create_info, None)?,
        //         )
        //     }
        // } else {
        //     None
        // };

        Ok(Self {
            inner : Arc::new( VulkanBufferInner{
                device_context: device_context.clone(),
                allocation_info: TrustCell::new(allocation_info),
                buffer_raw: Some(buffer_raw),
                buffer_def: buffer_def.clone(),
                // uniform_texel_view,
                // storage_texel_view,
            })
        })
    }
}

impl Drop for VulkanBufferInner {
    fn drop(&mut self) {
        log::trace!("destroying BufferVulkanInner");
        let _device = self.device_context.device();
        // if let Some(uniform_texel_view) = self.uniform_texel_view {
        //     unsafe {
        //         device.destroy_buffer_view(uniform_texel_view, None);
        //     }
        // }
        // if let Some(storage_texel_view) = self.storage_texel_view {
        //     unsafe {
        //         device.destroy_buffer_view(storage_texel_view, None);
        //     }
        // }

        if let Some(buffer_raw) = &self.buffer_raw {
            log::trace!(
                "Buffer {:?} destroying with size {} (always mapped: {:?})",
                buffer_raw.buffer,
                self.buffer_def.size,
                self.buffer_def.always_mapped
            );

            self.device_context
                .allocator()
                .destroy_buffer(buffer_raw.buffer, &buffer_raw.allocation);
        }

        log::trace!("destroyed BufferVulkanInner");
    }
}

impl Buffer<VulkanApi> for VulkanBuffer {
    fn buffer_def(&self) -> &BufferDef {
        &self.inner.buffer_def
    }

    fn map_buffer(&self) -> GfxResult<*mut u8> {
        let ptr = self
            .inner
            .device_context
            .allocator()
            .map_memory(&self.inner.buffer_raw.unwrap().allocation)?;
        *self.inner.allocation_info.borrow_mut() = self
            .inner
            .device_context
            .allocator()
            .get_allocation_info(&self.inner.buffer_raw.unwrap().allocation)?;
        Ok(ptr)
    }

    fn unmap_buffer(&self) -> GfxResult<()> {
        self.inner.device_context
            .allocator()
            .unmap_memory(&self.inner.buffer_raw.unwrap().allocation);
        *self.inner.allocation_info.borrow_mut() = self
            .inner
            .device_context
            .allocator()
            .get_allocation_info(&self.inner.buffer_raw.unwrap().allocation)?;
        Ok(())
    }

    fn mapped_memory(&self) -> Option<*mut u8> {
        let ptr = self.inner.allocation_info.borrow().get_mapped_data();
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    fn copy_to_host_visible_buffer<T: Copy>(&self, data: &[T]) -> GfxResult<()> {
        // Cannot check size of data == buffer because buffer size might be rounded up
        self.copy_to_host_visible_buffer_with_offset(data, 0)
    }

    fn copy_to_host_visible_buffer_with_offset<T: Copy>(
        &self,
        data: &[T],
        buffer_byte_offset: u64,
    ) -> GfxResult<()> {
        let data_size_in_bytes = legion_utils::memory::slice_size_in_bytes(data) as u64;
        assert!(buffer_byte_offset + data_size_in_bytes <= self.inner.buffer_def.size);

        let src = data.as_ptr().cast::<u8>();

        let required_alignment = std::mem::align_of::<T>();

        unsafe {
            let dst = self.map_buffer()?.add(buffer_byte_offset as usize);
            assert_eq!(((dst as usize) % required_alignment), 0);
            std::ptr::copy_nonoverlapping(src, dst, data_size_in_bytes as usize);
        }

        self.unmap_buffer()?;

        Ok(())
    }

    fn create_constant_buffer_view(&self, cbv_def: &crate::BufferViewDef) -> GfxResult<VulkanBufferView> {
        VulkanBufferView::from_buffer( &self, cbv_def)
    }
}
