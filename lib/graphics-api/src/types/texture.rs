use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicBool;
#[cfg(any(feature = "vulkan"))]
use std::sync::atomic::Ordering;

use crate::deferred_drop::Drc;
use crate::{
    DeviceContextDrc, Extents3D, GfxResult, TextureDef, TextureSubResource, TextureViewDef,
    TextureViewDrc,
};

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::{VulkanDeviceContext, VulkanRawImage, VulkanTexture};

#[derive(Debug)]
pub struct Texture {
    device_context: DeviceContextDrc,
    texture_def: TextureDef,
    is_undefined_layout: AtomicBool,
    texture_id: u32,

    #[cfg(feature = "vulkan")]
    platform_texture: VulkanTexture,
}

impl Drop for Texture {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_texture
            .destroy(&self.device_context.inner.platform_device_context);
    }
}

/// Holds the `vk::Image` and allocation as well as a few `vk::ImageViews` depending on the
/// provided `ResourceType` in the `texture_def`.
#[derive(Clone, Debug)]
pub struct TextureDrc {
    inner: Drc<Texture>,
}

impl PartialEq for TextureDrc {
    fn eq(&self, other: &Self) -> bool {
        self.inner.texture_id == other.inner.texture_id
    }
}

impl Eq for TextureDrc {}

impl Hash for TextureDrc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.texture_id.hash(state);
    }
}

impl TextureDrc {
    pub fn extents(&self) -> &Extents3D {
        &self.inner.texture_def.extents
    }

    pub fn array_length(&self) -> u32 {
        self.inner.texture_def.array_length
    }

    pub fn device_context(&self) -> &DeviceContextDrc {
        &self.inner.device_context
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_texture(&self) -> &VulkanTexture {
        &self.inner.platform_texture
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_device_context(&self) -> &VulkanDeviceContext {
        &self.inner.device_context.inner.platform_device_context
    }

    // Used internally as part of the hash for creating/reusing framebuffers
    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn texture_id(&self) -> u32 {
        self.inner.texture_id
    }

    // Command buffers check this to see if an image needs to be transitioned from UNDEFINED
    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn take_is_undefined_layout(&self) -> bool {
        self.inner
            .is_undefined_layout
            .swap(false, Ordering::Relaxed)
    }

    pub fn new(device_context: &DeviceContextDrc, texture_def: &TextureDef) -> GfxResult<Self> {
        Self::from_existing(
            device_context,
            #[cfg(feature = "vulkan")]
            None,
            texture_def,
        )
    }

    pub(crate) fn from_existing(
        device_context: &DeviceContextDrc,
        #[cfg(feature = "vulkan")] existing_image: Option<VulkanRawImage>,
        texture_def: &TextureDef,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let (platform_texture, texture_id) = VulkanTexture::from_existing(
            &device_context.inner.platform_device_context,
            #[cfg(feature = "vulkan")]
            existing_image,
            texture_def,
        )?;
        #[cfg(not(any(feature = "vulkan")))]
        let texture_id = 0;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(Texture {
                device_context: device_context.clone(),
                texture_def: *texture_def,
                is_undefined_layout: AtomicBool::new(true),
                texture_id,
                #[cfg(any(feature = "vulkan"))]
                platform_texture,
            }),
        })
    }

    pub fn definition(&self) -> &TextureDef {
        &self.inner.texture_def
    }

    pub fn map_texture(&self) -> GfxResult<TextureSubResource<'_>> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.inner
            .platform_texture
            .map_texture(&self.inner.device_context.inner.platform_device_context)
    }

    pub fn unmap_texture(&self) {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.inner
            .platform_texture
            .unmap_texture(&self.inner.device_context.inner.platform_device_context);
    }

    pub fn create_view(&self, view_def: &TextureViewDef) -> GfxResult<TextureViewDrc> {
        TextureViewDrc::new(self, view_def)
    }
}
