#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanTextureView;

use crate::{deferred_drop::Drc, GfxResult, TextureDrc, TextureViewDef};
#[cfg(any(feature = "vulkan"))]
use crate::{Descriptor, GPUViewType, ShaderResourceType};

#[derive(Clone, Debug)]
struct TextureView {
    definition: TextureViewDef,
    texture: TextureDrc,

    #[cfg(feature = "vulkan")]
    platform_texture_view: VulkanTextureView,
}

impl Drop for TextureView {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_texture_view
            .destroy(self.texture.platform_device_context());
    }
}

#[derive(Clone, Debug)]
pub struct TextureViewDrc {
    inner: Drc<TextureView>,
}

impl TextureViewDrc {
    pub(crate) fn new(texture: &TextureDrc, view_def: &TextureViewDef) -> GfxResult<Self> {
        let device_context = texture.device_context();

        #[cfg(feature = "vulkan")]
        let platform_texture_view = VulkanTextureView::new(texture, view_def).map_err(|e| {
            log::error!("Error creating platform texture view {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(TextureView {
                definition: *view_def,
                texture: texture.clone(),
                #[cfg(any(feature = "vulkan"))]
                platform_texture_view,
            }),
        })
    }

    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn definition(&self) -> &TextureViewDef {
        &self.inner.definition
    }

    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn texture(&self) -> &TextureDrc {
        &self.inner.texture
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_texture_view(&self) -> &VulkanTextureView {
        &self.inner.platform_texture_view
    }

    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn is_compatible_with_descriptor(&self, descriptor: &Descriptor) -> bool {
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
