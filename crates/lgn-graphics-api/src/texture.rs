use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use lgn_tracing::span_fn;

use crate::backends::{BackendRawImage, BackendTexture};
use crate::deferred_drop::Drc;
use crate::{
    DeviceContext, Extents3D, Format, GfxResult, MemoryUsage, PlaneSlice, ResourceFlags,
    ResourceUsage, TextureSubResource, TextureTiling, TextureView, TextureViewDef,
};

/// Used to create a `Texture`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextureDef {
    pub extents: Extents3D,
    pub array_length: u32,
    pub mip_count: u32,
    pub format: Format,
    pub usage_flags: ResourceUsage,
    pub resource_flags: ResourceFlags,
    pub mem_usage: MemoryUsage,
    pub tiling: TextureTiling,
}

impl Default for TextureDef {
    fn default() -> Self {
        Self {
            extents: Extents3D {
                width: 0,
                height: 0,
                depth: 0,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::UNDEFINED,
            usage_flags: ResourceUsage::empty(),
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        }
    }
}

impl TextureDef {
    pub fn is_2d(&self) -> bool {
        self.extents.depth == 1
    }

    pub fn is_3d(&self) -> bool {
        self.extents.depth > 1
    }

    pub fn is_cube(&self) -> bool {
        self.resource_flags.contains(ResourceFlags::TEXTURE_CUBE)
    }

    pub fn verify(&self) {
        assert!(self.extents.width > 0);
        assert!(self.extents.height > 0);
        assert!(self.extents.depth > 0);
        assert!(self.array_length > 0);
        assert!(self.mip_count > 0);

        assert!(!self
            .usage_flags
            .intersects(ResourceUsage::BUFFER_ONLY_USAGE_FLAGS));

        if self.resource_flags.contains(ResourceFlags::TEXTURE_CUBE) {
            assert_eq!(self.array_length % 6, 0);
        }

        // vdbdd: I think this validation is wrong
        assert!(
            !(self.format.has_depth()
                && self
                    .usage_flags
                    .intersects(ResourceUsage::AS_UNORDERED_ACCESS)),
            "Cannot use depth stencil as UAV"
        );
    }
}

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

    pub fn new(device_context: &DeviceContext, texture_def: &TextureDef) -> Self {
        Self::from_existing(device_context, None, texture_def)
    }

    pub(crate) fn from_existing(
        device_context: &DeviceContext,
        existing_image: Option<BackendRawImage>,
        texture_def: &TextureDef,
    ) -> Self {
        let (backend_texture, texture_id) =
            BackendTexture::from_existing(device_context, existing_image, texture_def);

        Self {
            inner: device_context.deferred_dropper().new_drc(TextureInner {
                device_context: device_context.clone(),
                texture_def: *texture_def,
                is_undefined_layout: AtomicBool::new(true),
                texture_id,
                backend_texture,
            }),
        }
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

    pub fn create_view(&self, view_def: &TextureViewDef) -> TextureView {
        TextureView::new(self, view_def)
    }
}
