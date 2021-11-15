use ash::vk;

use crate::backends::vulkan::VulkanDeviceContext;
use crate::{GfxResult, TextureDrc, TextureViewDef};

#[derive(Clone, Debug)]
pub(crate) struct VulkanTextureView {
    vk_image_view: vk::ImageView,
}

impl VulkanTextureView {
    pub(crate) fn new(texture: &TextureDrc, view_def: &TextureViewDef) -> GfxResult<Self> {
        view_def.verify(texture.definition());

        let device_context = texture.platform_device_context();
        let device = device_context.device();
        let texture_def = texture.definition();
        let aspect_mask = super::internal::image_format_to_aspect_mask(texture_def.format);
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(view_def.first_mip)
            .level_count(view_def.mip_count)
            .base_array_layer(view_def.first_array_slice)
            .layer_count(view_def.array_size);
        let builder = vk::ImageViewCreateInfo::builder()
            .image(texture.platform_texture().vk_image())
            .components(vk::ComponentMapping::default())
            .view_type(view_def.view_dimension.into())
            .format(texture_def.format.into())
            .subresource_range(subresource_range.build());
        let vk_image_view = unsafe { device.create_image_view(&builder.build(), None)? };

        Ok(Self { vk_image_view })
    }

    pub fn destroy(&self, device_context: &VulkanDeviceContext) {
        let device = device_context.device();
        unsafe {
            device.destroy_image_view(self.vk_image_view, None);
        }
    }

    pub fn vk_image_view(&self) -> vk::ImageView {
        self.vk_image_view
    }
}
