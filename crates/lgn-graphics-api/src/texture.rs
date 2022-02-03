use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use lgn_tracing::span_fn;

use crate::backends::{BackendRawImage, BackendTexture};
use crate::deferred_drop::Drc;
use crate::{
    DeviceContext, Extents3D, GfxResult, PlaneSlice, TextureDef, TextureSubResource, TextureView,
    TextureViewDef,
};

#[derive(Debug)]
pub(crate) struct TextureInner {
    pub(crate) device_context: DeviceContext,
    pub(crate) texture_def: TextureDef,
    pub(crate) is_undefined_layout: AtomicBool,
    pub(crate) texture_id: u32,
    pub(crate) backend_texture: BackendTexture,
}

impl Drop for TextureInner {
    fn drop(&mut self) {
        self.backend_texture.destroy(&self.device_context);
    }
}

/// Holds the `vk::Image` and allocation as well as a few `vk::ImageViews`
/// depending on the provided `ResourceType` in the `texture_def`.
#[derive(Clone, Debug)]
pub struct Texture {
    pub(crate) inner: Drc<TextureInner>,
}

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        self.inner.texture_id == other.inner.texture_id
    }
}

impl Eq for Texture {}

impl Hash for Texture {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.texture_id.hash(state);
    }
}

impl Texture {
    pub fn extents(&self) -> &Extents3D {
        &self.inner.texture_def.extents
    }

    pub fn array_length(&self) -> u32 {
        self.inner.texture_def.array_length
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    // // Used internally as part of the hash for creating/reusing framebuffers
    pub(crate) fn texture_id(&self) -> u32 {
        self.inner.texture_id
    }

    // Command buffers check this to see if an image needs to be transitioned from
    // UNDEFINED
    pub(crate) fn take_is_undefined_layout(&self) -> bool {
        self.inner
            .is_undefined_layout
            .swap(false, Ordering::Relaxed)
    }

    pub fn new(device_context: &DeviceContext, texture_def: &TextureDef) -> GfxResult<Self> {
        Self::from_existing(device_context, None, texture_def)
    }

    pub(crate) fn from_existing(
        device_context: &DeviceContext,
        existing_image: Option<BackendRawImage>,
        texture_def: &TextureDef,
    ) -> GfxResult<Self> {
        let (backend_texture, texture_id) =
            BackendTexture::from_existing(device_context, existing_image, texture_def)?;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(TextureInner {
                device_context: device_context.clone(),
                texture_def: *texture_def,
                is_undefined_layout: AtomicBool::new(true),
                texture_id,
                backend_texture,
            }),
        })
    }

    pub fn definition(&self) -> &TextureDef {
        &self.inner.texture_def
    }

    #[span_fn]
    pub fn map_texture(&self, plane: PlaneSlice) -> GfxResult<TextureSubResource<'_>> {
        self.backend_map_texture(plane)
    }

    pub fn unmap_texture(&self) {
        self.backend_unmap_texture();
    }

    pub fn create_view(&self, view_def: &TextureViewDef) -> GfxResult<TextureView> {
        TextureView::new(self, view_def)
    }
}
