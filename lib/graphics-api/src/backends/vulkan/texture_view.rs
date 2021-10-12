use std::sync::Arc;

use ash::vk;

use crate::{GfxResult, Texture, TextureView, TextureViewDef, VulkanApi};

use super::VulkanTexture;


#[derive(Clone, Debug)]
struct VulkanTextureViewInner {
    view_def : TextureViewDef,
    texture : VulkanTexture,
    vk_image_view : vk::ImageView
}

impl Drop for VulkanTextureViewInner {
    fn drop(&mut self) {
        let device = self.texture.device_context().device();
        unsafe {
            device.destroy_image_view(self.vk_image_view, None);
        }
    }
}

#[derive(Clone, Debug)]
pub struct VulkanTextureView {
    inner: Arc<VulkanTextureViewInner>    
}

impl TextureView<VulkanApi> for VulkanTextureView {
    fn texture(&self) -> &VulkanTexture {
        &self.inner.texture
    }

    fn view_def(&self) -> &TextureViewDef {
        &self.inner.view_def
    }
}

impl VulkanTextureView  {

    pub(super) fn new(texture: &VulkanTexture, view_def: &TextureViewDef) -> GfxResult<Self> {
        
        view_def.verify::<VulkanApi>(texture);

        let device = texture.device_context().device();
        let texture_def = texture.texture_def();

        let aspect_mask = super::internal::image_format_to_aspect_mask(texture_def.format);

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(view_def.first_mip)
            .level_count(view_def.mip_count)
            .base_array_layer(view_def.first_slice)
            .layer_count(view_def.slice_count);

        let builder = vk::ImageViewCreateInfo::builder()
            .image(texture.vk_image())
            .components(vk::ComponentMapping::default())
            .view_type(view_def.view_type.into())
            .format(texture_def.format.into())        
            .subresource_range(subresource_range.build());

        let vk_image_view = unsafe {
            device.create_image_view(&builder.build(), None)?
        };

        Ok( VulkanTextureView{
            inner: Arc::new( VulkanTextureViewInner {
                view_def: *view_def,
                texture: texture.clone(),
                vk_image_view
            })
        })
    }

    pub(super) fn vulkan_texture(&self) -> &VulkanTexture {
        &self.inner.texture
    }
    
    pub(super) fn view_def(&self) -> &TextureViewDef {
        &self.inner.view_def
    }

    pub(super) fn vk_image_view(&self) -> vk::ImageView {
        self.inner.vk_image_view
    }
}