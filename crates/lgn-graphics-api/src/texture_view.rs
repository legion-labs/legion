#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanTextureView;
use crate::{deferred_drop::Drc, GfxResult, Texture, TextureViewDef};
#[cfg(any(feature = "vulkan"))]
use crate::{Descriptor, GPUViewType, ShaderResourceType};

#[derive(Clone, Debug)]
pub(crate) struct TextureViewInner {
    definition: TextureViewDef,
    texture: Texture,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_texture_view: VulkanTextureView,
}

impl Drop for TextureViewInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_texture_view
            .destroy(self.texture.device_context());
    }
}

#[derive(Clone, Debug)]
pub struct TextureView {
    pub(crate) inner: Drc<TextureViewInner>,
}

impl TextureView {
    pub(crate) fn new(texture: &Texture, view_def: &TextureViewDef) -> GfxResult<Self> {
        let device_context = texture.device_context();

        #[cfg(feature = "vulkan")]
        let platform_texture_view = VulkanTextureView::new(texture, view_def).map_err(|e| {
            lgn_tracing::error!("Error creating platform texture view {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(TextureViewInner {
                definition: *view_def,
                texture: texture.clone(),
                #[cfg(any(feature = "vulkan"))]
                platform_texture_view,
            }),
        })
    }

    pub fn definition(&self) -> &TextureViewDef {
        &self.inner.definition
    }

    pub(crate) fn texture(&self) -> &Texture {
        &self.inner.texture
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
                self.inner.definition.gpu_view_type == GPUViewType::ShaderResource
                    && self.inner.definition.array_size == 1
            }

            ShaderResourceType::RWTexture2D
            | ShaderResourceType::RWTexture2DArray
            | ShaderResourceType::RWTexture3D => {
                self.inner.definition.gpu_view_type == GPUViewType::UnorderedAccess
                    && self.inner.definition.array_size == 1
            }
        }
    }
}
