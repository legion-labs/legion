use ash::vk;

use super::{VulkanDescriptor, VulkanTexture};
use crate::{
    backends::deferred_drop::Drc, GPUViewType, GfxResult, ShaderResourceType, Texture, TextureView,
    TextureViewDef, VulkanApi,
};

#[derive(Clone, Debug)]
struct VulkanTextureViewInner {
    definition: TextureViewDef,
    texture: VulkanTexture,
    vk_image_view: vk::ImageView,
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
    inner: Drc<VulkanTextureViewInner>,
}

impl TextureView<VulkanApi> for VulkanTextureView {
    fn definition(&self) -> &TextureViewDef {
        &self.inner.definition
    }

    fn texture(&self) -> &VulkanTexture {
        &self.inner.texture
    }
}

impl VulkanTextureView {
    pub(super) fn new(texture: &VulkanTexture, view_def: &TextureViewDef) -> GfxResult<Self> {
        view_def.verify(texture.definition());

        let device_context = texture.device_context();
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
            .image(texture.vk_image())
            .components(vk::ComponentMapping::default())
            .view_type(view_def.view_dimension.into())
            .format(texture_def.format.into())
            .subresource_range(subresource_range.build());
        let vk_image_view = unsafe { device.create_image_view(&builder.build(), None)? };

        Ok(Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(VulkanTextureViewInner {
                    definition: *view_def,
                    texture: texture.clone(),
                    vk_image_view,
                }),
        })
    }

    pub(super) fn vulkan_texture(&self) -> &VulkanTexture {
        &self.inner.texture
    }

    pub(super) fn view_def(&self) -> &TextureViewDef {
        &self.inner.definition
    }

    pub(super) fn vk_image_view(&self) -> vk::ImageView {
        self.inner.vk_image_view
    }

    pub(super) fn is_compatible_with_descriptor(&self, descriptor: &VulkanDescriptor) -> bool {
        match descriptor.shader_resource_type {
            ShaderResourceType::ConstantBuffer
            | ShaderResourceType::StructuredBuffer
            | ShaderResourceType::ByteAdressBuffer
            | ShaderResourceType::RWStructuredBuffer
            | ShaderResourceType::RWByteAdressBuffer
            | ShaderResourceType::Sampler => false,

            ShaderResourceType::Texture2D
            | ShaderResourceType::Texture3D
            | ShaderResourceType::Texture2DArray
            | ShaderResourceType::TextureCube
            | ShaderResourceType::TextureCubeArray => {
                self.inner.definition.gpu_view_type == GPUViewType::ShaderResourceView
                    && self.inner.definition.array_size == 1
            }

            ShaderResourceType::RWTexture2D
            | ShaderResourceType::RWTexture2DArray
            | ShaderResourceType::RWTexture3D => {
                self.inner.definition.gpu_view_type == GPUViewType::UnorderedAccessView
                    && self.inner.definition.array_size == 1
            }
        }
    }
}
