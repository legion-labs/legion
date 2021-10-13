use super::*;
use crate::backends::vulkan::{VulkanApi, VulkanDeviceContext, VulkanRenderpass};
use crate::*;
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

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
        color_targets: &[ColorRenderTargetBinding<'_, VulkanApi>],
        depth_target: Option<&DepthStencilRenderTargetBinding<'_, VulkanApi>>,
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
                .view_def()
                .first_mip
                .hash(&mut hasher);
            color_target
                .texture_view
                .view_def()
                .first_array_slice
                .hash(&mut hasher);

            // if let Some(resolve_target) = color_target.resolve_target {
            //     resolve_target.texture().texture_id().hash(&mut hasher);
            //     color_target.resolve_mip_slice.hash(&mut hasher);
            //     color_target.resolve_array_slice.hash(&mut hasher);
            // }
        }

        if let Some(depth_target) = &depth_target {
            depth_target
                .texture_view
                .texture()
                .texture_id()
                .hash(&mut hasher);
            depth_target
                .texture_view
                .view_def()
                .first_mip
                .hash(&mut hasher);
            depth_target
                .texture_view
                .view_def()
                .first_array_slice
                .hash(&mut hasher);
        }
        hasher.finish()
    }

    pub(crate) fn create_framebuffer(
        device_context: &VulkanDeviceContext,
        renderpass: &VulkanRenderpass,
        color_targets: &[ColorRenderTargetBinding<'_, VulkanApi>],
        depth_target: Option<&DepthStencilRenderTargetBinding<'_, VulkanApi>>,
    ) -> GfxResult<FramebufferVulkan> {
        let mut color_attachments = Vec::with_capacity(color_targets.len());
        // let mut resolve_attachments = Vec::with_capacity(color_targets.len());

        for color_target in color_targets {
            color_attachments.push(FramebufferVulkanAttachment {
                texture_view: color_target.texture_view.clone(),
                // array_slice: color_target.array_slice,
                // mip_slice: color_target.mip_slice,
            });

            // if let Some(resolve_target) = color_target.resolve_target {
            // resolve_attachments.push(FramebufferVulkanAttachment {
            // texture_view: resolve_target.clone(),
            // array_slice: color_target.resolve_array_slice,
            // mip_slice: color_target.resolve_mip_slice,
            // });
            // }
        }

        FramebufferVulkan::new(
            device_context,
            &FramebufferVulkanDef {
                renderpass: renderpass.clone(),
                color_attachments,
                // resolve_attachments,
                depth_stencil_attachment: depth_target.as_ref().map(|x| {
                    FramebufferVulkanAttachment {
                        texture_view: x.texture_view.clone(),
                        // array_slice: x.array_slice,
                        // mip_slice: x.mip_slice,
                    }
                }),
            },
        )
    }

    pub(crate) fn get_or_create_framebuffer(
        &mut self,
        device_context: &VulkanDeviceContext,
        renderpass: &VulkanRenderpass,
        color_targets: &[ColorRenderTargetBinding<'_, VulkanApi>],
        depth_target: Option<&DepthStencilRenderTargetBinding<'_, VulkanApi>>,
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
