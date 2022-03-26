use std::ptr::slice_from_raw_parts;
use std::sync::atomic::Ordering;

use ash::vk::{
    self, DeviceMemory, DeviceSize, ExportMemoryAllocateInfo, ExternalMemoryHandleTypeFlags,
    ExternalMemoryImageCreateInfo, ImageCreateInfo, ImageType, MemoryAllocateInfo,
    MemoryPropertyFlags,
};
use lgn_tracing::trace;

use crate::{
    DeviceContext, ExternalResourceHandle, GfxResult, MemoryUsage, PlaneSlice, ResourceFlags,
    ResourceUsage, Texture, TextureDef, TextureSubResource,
};

static NEXT_TEXTURE_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

// This is used to allow the underlying image/allocation to be removed from a
// VulkanTexture, or to init a VulkanTexture with an existing image/allocation.
// If the allocation is none, we will not destroy the image when VulkanRawImage
// is dropped
#[derive(Debug)]
pub(crate) struct VulkanRawImage {
    pub(crate) vk_image: vk::Image,
    pub(crate) vk_allocation: Option<vk_mem::Allocation>,
    pub(crate) vk_device_memory: Option<DeviceMemory>,
    pub(crate) vk_alloc_size: DeviceSize,
}

impl VulkanRawImage {
    fn destroy_image(&mut self, device_context: &DeviceContext) {
        if let Some(allocation) = self.vk_allocation.take() {
            trace!("destroying ImageVulkan");
            assert_ne!(self.vk_image, vk::Image::null());
            device_context
                .vk_allocator()
                .destroy_image(self.vk_image, &allocation);
            self.vk_image = vk::Image::null();
            trace!("destroyed ImageVulkan");
        } else if let Some(device_memory) = self.vk_device_memory.take() {
            unsafe {
                device_context.vk_device().free_memory(device_memory, None);
                device_context
                    .vk_device()
                    .destroy_image(self.vk_image, None);
            };
        } else {
            trace!("ImageVulkan has no allocation associated with it, not destroying image");
            self.vk_image = vk::Image::null();
        }
    }
}

impl Drop for VulkanRawImage {
    fn drop(&mut self) {
        assert!(self.vk_allocation.is_none());
    }
}

#[derive(Debug)]
pub(crate) struct VulkanTexture {
    pub image: VulkanRawImage,
    pub aspect_mask: vk::ImageAspectFlags,
}

impl VulkanTexture {
    // This path is mostly so we can wrap a provided swapchain image
    pub fn from_existing(
        device_context: &DeviceContext,
        existing_image: Option<VulkanRawImage>,
        texture_def: &TextureDef,
    ) -> (Self, u32) {
        texture_def.verify();

        let image_type = image_type_from_texture_def(texture_def);

        let is_cubemap = texture_def
            .resource_flags
            .contains(ResourceFlags::TEXTURE_CUBE);
        let format_vk = texture_def.format.into();

        // create the image
        let image = if let Some(existing_image) = existing_image {
            existing_image
        } else {
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

            //TODO: Could check vkGetPhysicalDeviceFormatProperties for if we support the
            // format for the various ways we might use it
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
                .usage(usage_flags_for_def(texture_def))
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .samples(vk::SampleCountFlags::TYPE_1) // texture_def.sample_count.into())
                .flags(create_flags);

            //let allocator = device.allocator().clone();
            let (image, allocation, allocation_info) = device_context
                .vk_allocator()
                .create_image(&image_create_info, &allocation_create_info)
                .unwrap();

            VulkanRawImage {
                vk_image: image,
                vk_allocation: Some(allocation),
                vk_device_memory: None,
                vk_alloc_size: allocation_info.get_size() as DeviceSize,
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
        let texture_id = NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed);

        (Self { image, aspect_mask }, texture_id)
    }

    pub fn new_export_capable(
        device_context: &DeviceContext,
        texture_def: &TextureDef,
    ) -> (Self, u32) {
        let extent = vk::Extent3D {
            width: texture_def.extents.width,
            height: texture_def.extents.height,
            depth: texture_def.extents.depth,
        };

        let mut image_create_info = ImageCreateInfo {
            image_type: image_type_from_texture_def(texture_def),
            format: texture_def.format.into(),
            extent,
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: texture_def.tiling.into(),
            usage: usage_flags_for_def(texture_def),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..ImageCreateInfo::default()
        };

        #[cfg(target_os = "windows")]
        let handle_type = ExternalMemoryHandleTypeFlags::OPAQUE_WIN32;

        #[cfg(target_os = "linux")]
        let handle_type = ExternalMemoryHandleTypeFlags::OPAQUE_FD;

        let mut ext_image_create_info = ExternalMemoryImageCreateInfo::default();
        if texture_def
            .usage_flags
            .intersects(ResourceUsage::AS_EXPORT_CAPABLE)
        {
            ext_image_create_info.handle_types |= handle_type;

            image_create_info.p_next =
                std::ptr::addr_of!(ext_image_create_info).cast::<std::ffi::c_void>();
        }

        let image = unsafe {
            device_context
                .vk_device()
                .create_image(&image_create_info, None)
                .unwrap()
        };

        let memory_requirements = unsafe {
            device_context
                .vk_device()
                .get_image_memory_requirements(image)
        };

        let mut memory_type_index: u32 = u32::MAX;
        let device_memory_properties = device_context.get_physical_device_memory_properties();
        for i in 0..device_memory_properties.memory_type_count {
            if (memory_requirements.memory_type_bits & (1 << i)) != 0
                && ((MemoryPropertyFlags::DEVICE_LOCAL
                    & device_memory_properties.memory_types[i as usize].property_flags)
                    == MemoryPropertyFlags::DEVICE_LOCAL)
            {
                memory_type_index = i;
            }
        }

        let mut memory_create_info = MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index,
            ..MemoryAllocateInfo::default()
        };

        let mut ext_memory_create_info = ExportMemoryAllocateInfo::default();
        if texture_def
            .usage_flags
            .intersects(ResourceUsage::AS_EXPORT_CAPABLE)
        {
            ext_memory_create_info.handle_types |= handle_type;

            memory_create_info.p_next =
                std::ptr::addr_of!(ext_memory_create_info).cast::<std::ffi::c_void>();
        }

        let memory = unsafe {
            device_context
                .vk_device()
                .allocate_memory(&memory_create_info, None)
                .unwrap()
        };

        unsafe {
            device_context
                .vk_device()
                .bind_image_memory(image, memory, 0)
                .unwrap();
        }

        let raw_image = VulkanRawImage {
            vk_image: image,
            vk_allocation: None,
            vk_device_memory: Some(memory),
            vk_alloc_size: memory_requirements.size,
        };

        let aspect_mask = super::internal::image_format_to_aspect_mask(texture_def.format);

        // Used for hashing framebuffers
        let texture_id = NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed);

        (
            Self {
                image: raw_image,
                aspect_mask,
            },
            texture_id,
        )
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) {
        self.image.destroy_image(device_context);
    }

    pub fn external_memory_handle(&self, device_context: &DeviceContext) -> ExternalResourceHandle {
        device_context.vk_external_memory_handle(self.image.vk_device_memory.unwrap())
    }
}

fn image_type_from_texture_def(texture_def: &TextureDef) -> ImageType {
    match texture_def.extents.depth {
        0 => panic!(),
        1 => vk::ImageType::TYPE_2D,
        2.. => vk::ImageType::TYPE_3D,
    }
}

fn usage_flags_for_def(texture_def: &TextureDef) -> vk::ImageUsageFlags {
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
        usage_flags |= vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST;
    }
    usage_flags
}

impl Texture {
    pub(crate) fn vk_aspect_mask(&self) -> vk::ImageAspectFlags {
        self.inner.backend_texture.aspect_mask
    }

    pub(crate) fn vk_image(&self) -> vk::Image {
        self.inner.backend_texture.image.vk_image
    }

    pub(crate) fn vk_allocation(&self) -> Option<vk_mem::Allocation> {
        self.inner.backend_texture.image.vk_allocation
    }

    pub fn vk_alloc_size(&self) -> DeviceSize {
        self.inner.backend_texture.image.vk_alloc_size
    }

    pub(crate) fn backend_map_texture(
        &self,
        plane: PlaneSlice,
    ) -> GfxResult<TextureSubResource<'_>> {
        let ptr = self
            .inner
            .device_context
            .vk_allocator()
            .map_memory(&self.vk_allocation().unwrap())?;

        let aspect_mask = match plane {
            crate::PlaneSlice::Default => {
                super::internal::image_format_to_aspect_mask(self.inner.texture_def.format)
            }
            crate::PlaneSlice::Depth => vk::ImageAspectFlags::DEPTH,
            crate::PlaneSlice::Stencil => vk::ImageAspectFlags::STENCIL,
            crate::PlaneSlice::Plane0 => vk::ImageAspectFlags::PLANE_0,
            crate::PlaneSlice::Plane1 => vk::ImageAspectFlags::PLANE_1,
            crate::PlaneSlice::Plane2 => vk::ImageAspectFlags::PLANE_2,
        };
        let sub_res = vk::ImageSubresource::builder()
            .aspect_mask(aspect_mask)
            .build();

        unsafe {
            let sub_res_layout = self
                .inner
                .device_context
                .vk_device()
                .get_image_subresource_layout(self.inner.backend_texture.image.vk_image, sub_res);

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

    pub(crate) fn backend_unmap_texture(&self) {
        self.inner
            .device_context
            .vk_allocator()
            .unmap_memory(&self.vk_allocation().unwrap());
    }
}
