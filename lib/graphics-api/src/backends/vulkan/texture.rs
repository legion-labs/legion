use super::{VulkanApi, VulkanDeviceContext};
use crate::{
    Extents3D, GfxResult, MemoryUsage, ResourceType, Texture, TextureDef, TextureDimensions,
    TextureSubResource,
};
use ash::vk;
use std::hash::{Hash, Hasher};
use std::ptr::slice_from_raw_parts;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// This is used to allow the underlying image/allocation to be removed from a VulkanTexture,
// or to init a VulkanTexture with an existing image/allocation. If the allocation is none, we
// will not destroy the image when VulkanRawImage is dropped
#[derive(Debug)]
pub struct VulkanRawImage {
    pub image: vk::Image,
    pub allocation: Option<vk_mem::Allocation>,
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
pub struct TextureVulkanInner {
    device_context: VulkanDeviceContext,
    texture_def: TextureDef,
    image: VulkanRawImage,
    aspect_mask: vk::ImageAspectFlags,

    // For reading
    srv_view: Option<vk::ImageView>,
    srv_view_stencil: Option<vk::ImageView>,

    // For writing
    uav_views: Vec<vk::ImageView>,

    // RT
    is_undefined_layout: AtomicBool,
    texture_id: u32,
    render_target_view: Option<vk::ImageView>,
    render_target_view_slices: Vec<vk::ImageView>,
}

impl Drop for TextureVulkanInner {
    fn drop(&mut self) {
        let device = self.device_context.device();

        unsafe {
            if let Some(srv_view) = self.srv_view {
                device.destroy_image_view(srv_view, None);
            }

            if let Some(srv_view_stencil) = self.srv_view_stencil {
                device.destroy_image_view(srv_view_stencil, None);
            }

            for uav_view in &self.uav_views {
                device.destroy_image_view(*uav_view, None);
            }

            if let Some(render_target_view) = self.render_target_view {
                device.destroy_image_view(render_target_view, None);
            }

            for view_slice in &self.render_target_view_slices {
                device.destroy_image_view(*view_slice, None);
            }
        }

        self.image.destroy_image(&self.device_context);
    }
}

/// Holds the `vk::Image` and allocation as well as a few `vk::ImageViews` depending on the
/// provided `ResourceType` in the `texture_def`.
#[derive(Clone, Debug)]
pub struct VulkanTexture {
    pub(super) inner: Arc<TextureVulkanInner>,
}

impl PartialEq for VulkanTexture {
    fn eq(&self, other: &Self) -> bool {
        self.inner.texture_id == other.inner.texture_id
    }
}

impl Eq for VulkanTexture {}

impl Hash for VulkanTexture {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.texture_id.hash(state);
    }
}

impl VulkanTexture {
    pub fn extents(&self) -> &Extents3D {
        &self.inner.texture_def.extents
    }

    pub fn array_length(&self) -> u32 {
        self.inner.texture_def.array_length
    }

    pub fn vk_aspect_mask(&self) -> vk::ImageAspectFlags {
        self.inner.aspect_mask
    }

    pub fn vk_image(&self) -> vk::Image {
        self.inner.image.image
    }

    pub fn vk_allocation(&self) -> Option<vk_mem::Allocation> {
        self.inner.image.allocation
    }

    pub fn device_context(&self) -> &VulkanDeviceContext {
        &self.inner.device_context
    }

    // Color/Depth
    pub fn vk_srv_view(&self) -> Option<vk::ImageView> {
        self.inner.srv_view
    }

    // Stencil-only
    pub fn vk_srv_view_stencil(&self) -> Option<vk::ImageView> {
        self.inner.srv_view_stencil
    }

    // Mip chain
    pub fn vk_uav_views(&self) -> &[vk::ImageView] {
        &self.inner.uav_views
    }

    pub fn render_target_vk_view(&self) -> Option<vk::ImageView> {
        self.inner.render_target_view
    }

    pub fn render_target_slice_vk_view(
        &self,
        depth: u32,
        array_index: u16,
        mip_level: u8,
    ) -> vk::ImageView {
        assert!(
            depth == 0
                || self
                    .inner
                    .texture_def
                    .resource_type
                    .intersects(ResourceType::RENDER_TARGET_DEPTH_SLICES)
        );
        assert!(
            array_index == 0
                || self
                    .inner
                    .texture_def
                    .resource_type
                    .intersects(ResourceType::RENDER_TARGET_ARRAY_SLICES)
        );

        let def = &self.inner.texture_def;
        let index = (mip_level as usize * def.array_length as usize * def.extents.depth as usize)
            + (array_index as usize * def.extents.depth as usize)
            + depth as usize;
        self.inner.render_target_view_slices[index]
    }

    // Used internally as part of the hash for creating/reusing framebuffers
    pub(crate) fn texture_id(&self) -> u32 {
        self.inner.texture_id
    }

    // Command buffers check this to see if an image needs to be transitioned from UNDEFINED
    pub(crate) fn take_is_undefined_layout(&self) -> bool {
        self.inner
            .is_undefined_layout
            .swap(false, Ordering::Relaxed)
    }

    pub fn new(device_context: &VulkanDeviceContext, texture_def: &TextureDef) -> GfxResult<Self> {
        Self::from_existing(device_context, None, texture_def)
    }

    // This path is mostly so we can wrap a provided swapchain image
    pub fn from_existing(
        device_context: &VulkanDeviceContext,
        existing_image: Option<VulkanRawImage>,
        texture_def: &TextureDef,
    ) -> GfxResult<Self> {
        texture_def.verify();

        // if RW texture, create image viewsper mip, otherwise none?

        //
        // Determine desired image type
        //
        let dimensions = texture_def
            .dimensions
            .determine_dimensions(texture_def.extents);
        let image_type = match dimensions {
            TextureDimensions::Dim1D => vk::ImageType::TYPE_1D,
            TextureDimensions::Dim2D => vk::ImageType::TYPE_2D,
            TextureDimensions::Dim3D => vk::ImageType::TYPE_3D,
            TextureDimensions::Auto => panic!("dimensions() should not return auto"),
        };

        let is_cubemap = texture_def
            .resource_type
            .contains(ResourceType::TEXTURE_CUBE);
        let format_vk = texture_def.format.into();

        // create the image
        let image = if let Some(existing_image) = existing_image {
            existing_image
        } else {
            //
            // Determine image usage flags
            //
            let mut usage_flags =
                super::util::resource_type_image_usage_flags(texture_def.resource_type);
            if texture_def
                .resource_type
                .intersects(ResourceType::RENDER_TARGET_COLOR)
            {
                usage_flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
            } else if texture_def
                .resource_type
                .intersects(ResourceType::RENDER_TARGET_DEPTH_STENCIL)
            {
                usage_flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
            }

            if usage_flags.intersects(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE) {
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
                .samples(texture_def.sample_count.into())
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

        let mut image_view_type = if image_type == vk::ImageType::TYPE_1D {
            if texture_def.array_length > 1 {
                vk::ImageViewType::TYPE_1D_ARRAY
            } else {
                vk::ImageViewType::TYPE_1D
            }
        } else if image_type == vk::ImageType::TYPE_2D {
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
        let aspect_mask = super::util::image_format_to_aspect_mask(texture_def.format);
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

            let rt_image_view_type = if texture_def.dimensions != TextureDimensions::Dim1D {
                if depth_array_size_multiple > 1 {
                    vk::ImageViewType::TYPE_2D_ARRAY
                } else {
                    vk::ImageViewType::TYPE_2D
                }
            } else if depth_array_size_multiple > 1 {
                vk::ImageViewType::TYPE_1D_ARRAY
            } else {
                vk::ImageViewType::TYPE_1D
            };

            //SRV
            let aspect_mask = super::util::image_format_to_aspect_mask(texture_def.format);
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

        // Used for hashing framebuffers
        let texture_id = crate::backends::shared::NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed);

        let inner = TextureVulkanInner {
            texture_def: texture_def.clone(),
            device_context: device_context.clone(),
            image,
            aspect_mask,
            srv_view,
            srv_view_stencil,
            uav_views,
            texture_id,
            render_target_view,
            render_target_view_slices,
            is_undefined_layout: AtomicBool::new(true),
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

impl Texture<VulkanApi> for VulkanTexture {
    fn texture_def(&self) -> &TextureDef {
        &self.inner.texture_def
    }
    fn map_texture(&self) -> GfxResult<TextureSubResource<'_>> {
        let ptr = self
            .inner
            .device_context
            .allocator()
            .map_memory(&self.vk_allocation().unwrap())?;

        let sub_res = vk::ImageSubresource::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .build();

        unsafe {
            let sub_res_layout = self
                .inner
                .device_context
                .device()
                .get_image_subresource_layout(self.inner.image.image, sub_res);

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

    fn unmap_texture(&self) -> GfxResult<()> {
        self.inner
            .device_context
            .allocator()
            .unmap_memory(&self.vk_allocation().unwrap());

        Ok(())
    }
}
