use std::hash::{Hash, Hasher};

use fnv::FnvHasher;

use super::LruCache;
use super::{
    VulkanRenderpassColorAttachment, VulkanRenderpassDef, VulkanRenderpassDepthAttachment,
};
use crate::backends::vulkan::{VulkanApi, VulkanDeviceContext, VulkanRenderpass};
use crate::{
    ColorRenderTargetBinding, DepthStencilRenderTargetBinding, GfxResult, Texture, TextureView,
};

pub(crate) struct VulkanRenderpassCache {
    cache: LruCache<VulkanRenderpass>,
}

impl VulkanRenderpassCache {
    pub(crate) fn new(max_count: usize) -> Self {
        Self {
            cache: LruCache::new(max_count),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.cache.clear();
    }

    pub(crate) fn renderpass_hash(
        color_targets: &[ColorRenderTargetBinding<'_, VulkanApi>],
        depth_target: Option<&DepthStencilRenderTargetBinding<'_, VulkanApi>>,
    ) -> u64 {
        let mut hasher = FnvHasher::default();
        for color_target in color_targets {
            let texture_def = color_target.texture_view.texture().definition();
            texture_def.format.hash(&mut hasher);
            color_target.clear_value.hash(&mut hasher);
            color_target.load_op.hash(&mut hasher);
        }

        if let Some(depth_target) = &depth_target {
            let texture_def = depth_target.texture_view.texture().definition();
            texture_def.format.hash(&mut hasher);
            depth_target.clear_value.hash(&mut hasher);
            depth_target.stencil_load_op.hash(&mut hasher);
            depth_target.depth_load_op.hash(&mut hasher);
        }
        hasher.finish()
    }

    pub(crate) fn create_renderpass(
        device_context: &VulkanDeviceContext,
        color_targets: &[ColorRenderTargetBinding<'_, VulkanApi>],
        depth_target: Option<&DepthStencilRenderTargetBinding<'_, VulkanApi>>,
    ) -> GfxResult<VulkanRenderpass> {
        let color_attachments: Vec<_> = color_targets
            .iter()
            .map(|x| VulkanRenderpassColorAttachment {
                format: x.texture_view.texture().definition().format,
                load_op: x.load_op,
                store_op: x.store_op,
            })
            .collect();

        let depth_attachment = depth_target
            .as_ref()
            .map(|x| VulkanRenderpassDepthAttachment {
                format: x.texture_view.texture().definition().format,
                depth_load_op: x.depth_load_op,
                stencil_load_op: x.stencil_load_op,
                depth_store_op: x.depth_store_op,
                stencil_store_op: x.stencil_store_op,
            });

        VulkanRenderpass::new(
            device_context,
            &VulkanRenderpassDef {
                color_attachments,
                depth_attachment,
            },
        )
    }

    pub(crate) fn get_or_create_renderpass(
        &mut self,
        device_context: &VulkanDeviceContext,
        color_targets: &[ColorRenderTargetBinding<'_, VulkanApi>],
        depth_target: Option<&DepthStencilRenderTargetBinding<'_, VulkanApi>>,
    ) -> GfxResult<VulkanRenderpass> {
        //
        // Hash it
        //
        let hash = Self::renderpass_hash(color_targets, depth_target);

        self.cache.get_or_create(hash, || {
            Self::create_renderpass(device_context, color_targets, depth_target)
        })
    }
}
