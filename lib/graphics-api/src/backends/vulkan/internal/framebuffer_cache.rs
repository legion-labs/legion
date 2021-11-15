use std::hash::{Hash, Hasher};

use fnv::FnvHasher;

use super::{FramebufferVulkan, FramebufferVulkanAttachment, FramebufferVulkanDef, LruCache};
use crate::backends::vulkan::VulkanRenderpass;
use crate::{
    ColorRenderTargetBinding, DepthStencilRenderTargetBinding, DeviceContext, GfxResult,
};

pub(crate) struct VulkanFramebufferCache {
    cache: LruCache<FramebufferVulkan>,
}

impl VulkanFramebufferCache {
    pub(crate) fn new(max_count: usize) -> Self {
        Self {
            cache: LruCache::new(max_count),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.cache.clear();
    }

    pub(crate) fn framebuffer_hash(
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: Option<&DepthStencilRenderTargetBinding<'_>>,
    ) -> u64 {
        let mut hasher = FnvHasher::default();
        for color_target in color_targets {
            color_target
                .texture_view
                .texture()
                .texture_id()
                .hash(&mut hasher);
            color_target
                .texture_view
                .definition()
                .first_mip
                .hash(&mut hasher);
            color_target
                .texture_view
                .definition()
                .first_array_slice
                .hash(&mut hasher);
        }

        if let Some(depth_target) = &depth_target {
            depth_target
                .texture_view
                .texture()
                .texture_id()
                .hash(&mut hasher);
            depth_target
                .texture_view
                .definition()
                .first_mip
                .hash(&mut hasher);
            depth_target
                .texture_view
                .definition()
                .first_array_slice
                .hash(&mut hasher);
        }
        hasher.finish()
    }

    pub(crate) fn create_framebuffer(
        device_context: &DeviceContext,
        renderpass: &VulkanRenderpass,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: Option<&DepthStencilRenderTargetBinding<'_>>,
    ) -> GfxResult<FramebufferVulkan> {
        let mut color_attachments = Vec::with_capacity(color_targets.len());

        for color_target in color_targets {
            color_attachments.push(FramebufferVulkanAttachment {
                texture_view: color_target.texture_view.clone(),
            });
        }

        FramebufferVulkan::new(
            device_context,
            &FramebufferVulkanDef {
                renderpass: renderpass.clone(),
                color_attachments,
                depth_stencil_attachment: depth_target.as_ref().map(|x| {
                    FramebufferVulkanAttachment {
                        texture_view: x.texture_view.clone(),
                    }
                }),
            },
        )
    }

    pub(crate) fn get_or_create_framebuffer(
        &mut self,
        device_context: &DeviceContext,
        renderpass: &VulkanRenderpass,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: Option<&DepthStencilRenderTargetBinding<'_>>,
    ) -> GfxResult<FramebufferVulkan> {
        //
        // Hash it
        //
        let hash = Self::framebuffer_hash(color_targets, depth_target);

        self.cache.get_or_create(hash, || {
            Self::create_framebuffer(device_context, renderpass, color_targets, depth_target)
        })
    }
}
