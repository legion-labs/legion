use std::ptr::slice_from_raw_parts;
use std::sync::atomic::Ordering;

use ash::vk::{self};

use crate::{GfxResult, MemoryUsage, ResourceFlags, ResourceUsage, TextureDef, TextureSubResource};

use super::VulkanDeviceContext;

// This is used to allow the underlying image/allocation to be removed from a VulkanTexture,
// or to init a VulkanTexture with an existing image/allocation. If the allocation is none, we
// will not destroy the image when VulkanRawImage is dropped
#[derive(Debug)]
pub(crate) struct VulkanRawImage {
    pub(crate) image: vk::Image,
    pub(crate) allocation: Option<vk_mem::Allocation>,
}

impl VulkanRawImage {
    fn destroy_image(&mut self, device_context: &VulkanDeviceContext) {
        if let Some(allocation) = self.allocation.take() {
            log::trace!("destroying ImageVulkan");
            assert_ne!(self.image, vk::Image::null());
            device_context
                .allocator()
                .destroy_image(self.image, &allocation);
            self.image = vk::Image::null();
            log::trace!("destroyed ImageVulkan");
        } else {
            log::trace!("ImageVulkan has no allocation associated with it, not destroying image");
            self.image = vk::Image::null();
        }
    }
}

impl Drop for VulkanRawImage {
    fn drop(&mut self) {
        assert!(self.allocation.is_none());
    }
}

#[derive(Debug)]
pub(crate) struct VulkanTexture {
    pub image: VulkanRawImage,
    pub aspect_mask: vk::ImageAspectFlags,
}

impl VulkanTexture {
    pub(super) fn vk_aspect_mask(&self) -> vk::ImageAspectFlags {
        self.aspect_mask
    }

    pub(super) fn vk_image(&self) -> vk::Image {
        self.image.image
    }

    pub(super) fn vk_allocation(&self) -> Option<vk_mem::Allocation> {
        self.image.allocation
    }

    // This path is mostly so we can wrap a provided swapchain image
    #[allow(clippy::too_many_lines)]
    pub fn from_existing(
        device_context: &VulkanDeviceContext,
        existing_image: Option<VulkanRawImage>,
        texture_def: &TextureDef,
    ) -> GfxResult<(Self, u32)> {
        texture_def.verify();
        //
        // Determine desired image type
        //
        let image_type = match texture_def.extents.depth {
            0 => panic!(),
            1 => vk::ImageType::TYPE_2D,
            2.. => vk::ImageType::TYPE_3D,
        };

        let is_cubemap = texture_def
            .resource_flags
            .contains(ResourceFlags::TEXTURE_CUBE);
        let format_vk = texture_def.format.into();

        // create the image
        let image = if let Some(existing_image) = existing_image {
            existing_image
        } else {
            //
            // Determine image usage flags
            //
            let mut usage_flags = vk::ImageUsageFlags::empty();
            if texture_def
                .usage_flags
                .intersects(ResourceUsage::AS_SHADER_RESOURCE)
            {
                usage_flags |= vk::ImageUsageFlags::SAMPLED;
            }
            if texture_def
                .usage_flags
                .intersects(ResourceUsage::AS_UNORDERED_ACCESS)
            {
                usage_flags |= vk::ImageUsageFlags::STORAGE;
            }
            if texture_def
                .usage_flags
                .intersects(ResourceUsage::AS_RENDER_TARGET)
            {
                usage_flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
            }
            if texture_def
                .usage_flags
                .intersects(ResourceUsage::AS_DEPTH_STENCIL)
            {
                usage_flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
            }
            if texture_def
                .usage_flags
                .intersects(ResourceUsage::AS_TRANSFERABLE)
            {
                usage_flags |=
                    vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST;
            }
            //
            // Determine image create flags
            //
            let mut create_flags = vk::ImageCreateFlags::empty();
            if is_cubemap {
                create_flags |= vk::ImageCreateFlags::CUBE_COMPATIBLE;
            }
            if image_type == vk::ImageType::TYPE_3D {
                create_flags |= vk::ImageCreateFlags::TYPE_2D_ARRAY_COMPATIBLE_KHR;
            }
            let required_flags = if texture_def.mem_usage != MemoryUsage::GpuOnly {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            } else {
                vk::MemoryPropertyFlags::empty()
            };

            //TODO: Could check vkGetPhysicalDeviceFormatProperties for if we support the format for
            // the various ways we might use it
            let allocation_create_info = vk_mem::AllocationCreateInfo {
                usage: texture_def.mem_usage.into(),
                flags: vk_mem::AllocationCreateFlags::NONE,
                required_flags,
                preferred_flags: vk::MemoryPropertyFlags::empty(),
                memory_type_bits: 0, // Do not exclude any memory types
                pool: None,
                user_data: None,
            };

            let extent = vk::Extent3D {
                width: texture_def.extents.width,
                height: texture_def.extents.height,
                depth: texture_def.extents.depth,
            };

            let image_create_info = vk::ImageCreateInfo::builder()
                .image_type(image_type)
                .extent(extent)
                .mip_levels(texture_def.mip_count)
                .array_layers(texture_def.array_length)
                .format(format_vk)
                .tiling(texture_def.tiling.into())
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usage(usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .samples(vk::SampleCountFlags::TYPE_1) // texture_def.sample_count.into())
                .flags(create_flags);

            //let allocator = device.allocator().clone();
            let (image, allocation, _allocation_info) = device_context
                .allocator()
                .create_image(&image_create_info, &allocation_create_info)
                .map_err(|_e| {
                    log::error!("Error creating image");
                    vk::Result::ERROR_UNKNOWN
                })?;

            VulkanRawImage {
                image,
                allocation: Some(allocation),
            }
        };

        let aspect_mask = super::internal::image_format_to_aspect_mask(texture_def.format);

        // VIEWS <<<<
        /*
        let mut image_view_type = if image_type == vk::ImageType::TYPE_2D {
            if is_cubemap {
                if texture_def.array_length > 6 {
                    vk::ImageViewType::CUBE_ARRAY
                } else {
                    vk::ImageViewType::CUBE
                }
            } else if texture_def.array_length > 1 {
                vk::ImageViewType::TYPE_2D_ARRAY
            } else {
                vk::ImageViewType::TYPE_2D
            }
        } else {
            assert_eq!(image_type, vk::ImageType::TYPE_3D);
            assert_eq!(texture_def.array_length, 1);
            vk::ImageViewType::TYPE_3D
        };

        //SRV
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_array_layer(0)
            .layer_count(texture_def.array_length)
            .base_mip_level(0)
            .level_count(texture_def.mip_count);

        let mut image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image.image)
            .view_type(image_view_type)
            .format(format_vk)
            .components(vk::ComponentMapping::default())
            .subresource_range(*subresource_range);

        // Create SRV without stencil
        let srv_view = if texture_def.resource_type.intersects(ResourceType::TEXTURE) {
            image_view_create_info.subresource_range.aspect_mask &= !vk::ImageAspectFlags::STENCIL;
            unsafe {
                Some(
                    device_context
                        .device()
                        .create_image_view(&*image_view_create_info, None)?,
                )
            }
        } else {
            None
        };

        // Create stencil-only SRV
        let srv_view_stencil = if texture_def
            .resource_type
            .intersects(ResourceType::TEXTURE_READ_WRITE)
            && aspect_mask.intersects(vk::ImageAspectFlags::STENCIL)
        {
            image_view_create_info.subresource_range.aspect_mask = vk::ImageAspectFlags::STENCIL;
            unsafe {
                Some(
                    device_context
                        .device()
                        .create_image_view(&*image_view_create_info, None)?,
                )
            }
        } else {
            None
        };

        // UAV
        let uav_views = if texture_def
            .resource_type
            .intersects(ResourceType::TEXTURE_READ_WRITE)
        {
            if image_view_type == vk::ImageViewType::CUBE_ARRAY
                || image_view_type == vk::ImageViewType::CUBE
            {
                image_view_type = vk::ImageViewType::TYPE_2D_ARRAY;
            }

            image_view_create_info.view_type = image_view_type;
            image_view_create_info.subresource_range.level_count = 1;

            let mut uav_views = Vec::with_capacity(texture_def.mip_count as usize);
            for i in 0..texture_def.mip_count {
                image_view_create_info.subresource_range.base_mip_level = i;
                unsafe {
                    uav_views.push(
                        device_context
                            .device()
                            .create_image_view(&*image_view_create_info, None)?,
                    );
                }
            }

            uav_views
        } else {
            vec![]
        };

        let mut render_target_view = None;
        let mut render_target_view_slices = vec![];
        if texture_def.resource_type.is_render_target() {
            // Render Target
            let depth_array_size_multiple = texture_def.extents.depth * texture_def.array_length;

            let rt_image_view_type = {
                if depth_array_size_multiple > 1 {
                    vk::ImageViewType::TYPE_2D_ARRAY
                } else {
                    vk::ImageViewType::TYPE_2D
                }
            };

            //SRV
            let aspect_mask = super::internal::image_format_to_aspect_mask(texture_def.format);
            let format_vk = texture_def.format.into();
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect_mask)
                .base_array_layer(0)
                .layer_count(depth_array_size_multiple)
                .base_mip_level(0)
                .level_count(1);

            let mut image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(image.image)
                .view_type(rt_image_view_type)
                .format(format_vk)
                .components(vk::ComponentMapping::default())
                .subresource_range(*subresource_range);

            render_target_view = Some(unsafe {
                device_context
                    .device()
                    .create_image_view(&*image_view_create_info, None)?
            });

            let array_or_depth_slices = texture_def.resource_type.intersects(
                ResourceType::RENDER_TARGET_ARRAY_SLICES | ResourceType::RENDER_TARGET_DEPTH_SLICES,
            );

            for i in 0..texture_def.mip_count {
                image_view_create_info.subresource_range.base_mip_level = i;

                if array_or_depth_slices {
                    for j in 0..depth_array_size_multiple {
                        image_view_create_info.subresource_range.layer_count = 1;
                        image_view_create_info.subresource_range.base_array_layer = j;
                        let view = unsafe {
                            device_context
                                .device()
                                .create_image_view(&*image_view_create_info, None)?
                        };
                        render_target_view_slices.push(view);
                    }
                } else {
                    let view = unsafe {
                        device_context
                            .device()
                            .create_image_view(&*image_view_create_info, None)?
                    };
                    render_target_view_slices.push(view);
                }
            }
        }
        */
        // VIEWS >>>>>

        // Used for hashing framebuffers
        let texture_id = crate::backends::shared::NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed);

        Ok((Self { image, aspect_mask }, texture_id))
    }

    pub fn destroy(&mut self, device_context: &VulkanDeviceContext) {
        self.image.destroy_image(device_context);
    }

    pub fn map_texture(
        &self,
        device_context: &VulkanDeviceContext,
    ) -> GfxResult<TextureSubResource<'_>> {
        let ptr = device_context
            .allocator()
            .map_memory(&self.vk_allocation().unwrap())?;

        let sub_res = vk::ImageSubresource::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .build();

        unsafe {
            let sub_res_layout = device_context
                .device()
                .get_image_subresource_layout(self.image.image, sub_res);

            Ok(TextureSubResource {
                data: &*slice_from_raw_parts(
                    ptr.add(sub_res_layout.offset as usize),
                    sub_res_layout.size as usize,
                ),
                row_pitch: sub_res_layout.row_pitch as u32,
                array_pitch: sub_res_layout.array_pitch as u32,
                depth_pitch: sub_res_layout.depth_pitch as u32,
            })
        }
    }

    pub fn unmap_texture(&self, device_context: &VulkanDeviceContext) {
        device_context
            .allocator()
            .unmap_memory(&self.vk_allocation().unwrap());
    }
}
