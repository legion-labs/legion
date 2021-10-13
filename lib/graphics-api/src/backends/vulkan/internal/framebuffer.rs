use crate::{GfxError, GfxResult, Texture};

use super::super::{VulkanDeviceContext, VulkanRenderpass, VulkanTextureView};
use ash::vk;
use std::sync::Arc;

pub(crate) struct FramebufferVulkanAttachment {
    // pub(crate) texture: VulkanTexture,
    // pub(crate) array_slice: Option<u16>,
    // pub(crate) mip_slice: Option<u8>,
    pub(crate) texture_view: VulkanTextureView,
}

pub(crate) struct FramebufferVulkanDef {
    pub(crate) renderpass: VulkanRenderpass,
    pub(crate) color_attachments: Vec<FramebufferVulkanAttachment>,
    // pub(crate) resolve_attachments: Vec<FramebufferVulkanAttachment>,
    pub(crate) depth_stencil_attachment: Option<FramebufferVulkanAttachment>,
}

pub(crate) struct FramebufferVulkanInner {
    device_context: VulkanDeviceContext,
    framebuffer: vk::Framebuffer,
    width: u32,
    height: u32,
}

impl Drop for FramebufferVulkanInner {
    fn drop(&mut self) {
        unsafe {
            self.device_context
                .device()
                .destroy_framebuffer(self.framebuffer, None);
        }
    }
}

#[derive(Clone)]
pub(crate) struct FramebufferVulkan {
    inner: Arc<FramebufferVulkanInner>,
}

impl FramebufferVulkan {
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    pub fn height(&self) -> u32 {
        self.inner.height
    }

    pub fn vk_framebuffer(&self) -> vk::Framebuffer {
        self.inner.framebuffer
    }

    pub fn new(
        device_context: &VulkanDeviceContext,
        framebuffer_def: &FramebufferVulkanDef,
    ) -> GfxResult<Self> {
        let (extents, array_length) =
            if let Some(first_color_rt) = framebuffer_def.color_attachments.first() {
                let texture_def = first_color_rt.texture_view.vulkan_texture().texture_def();
                let view_def = first_color_rt.texture_view.view_def();
                let extents = texture_def.extents;
                (extents, view_def.array_size)
            } else if let Some(depth_rt) = &framebuffer_def.depth_stencil_attachment {
                let texture_def = depth_rt.texture_view.vulkan_texture().texture_def();
                let view_def = depth_rt.texture_view.view_def();
                let extents = texture_def.extents;
                (extents, view_def.array_size)
            } else {
                return Err(GfxError::StringError(
                    "No render target in framebuffer def".to_string(),
                ));
            };

        let mut image_views = Vec::with_capacity(framebuffer_def.color_attachments.len() + 1);

        for color_rt in &framebuffer_def.color_attachments {
            let image_view = color_rt.texture_view.vk_image_view();
            // todo(vdbdd)
            // let image_view = if color_rt.array_slice.is_none() && color_rt.mip_slice.is_none() {
            //     color_rt.texture.render_target_vk_view().unwrap()
            // } else {
            //     color_rt.texture.render_target_slice_vk_view(
            //         0,
            //         color_rt.array_slice.unwrap_or(0),
            //         color_rt.mip_slice.unwrap_or(0),
            //     )
            // };
            image_views.push(image_view);
        }

        // for resolve_rt in &framebuffer_def.resolve_attachments {
        // let image_view = resolve_rt.texture_view.vk_image_view();
        // todo(vdbdd)
        // let image_view = if resolve_rt.array_slice.is_none() && resolve_rt.mip_slice.is_none() {
        //     resolve_rt.texture.render_target_vk_view().unwrap()
        // } else {
        //     resolve_rt.texture.render_target_slice_vk_view(
        //         0,
        //         resolve_rt.array_slice.unwrap_or(0),
        //         resolve_rt.mip_slice.unwrap_or(0),
        //     )
        // };
        // image_views.push(image_view);
        // }

        if let Some(depth_rt) = &framebuffer_def.depth_stencil_attachment {
            let image_view = depth_rt.texture_view.vk_image_view();
            // todo(vdbdd)
            // let image_view = if depth_rt.mip_slice.is_none() && depth_rt.array_slice.is_none() {
            //     depth_rt.texture.render_target_vk_view().unwrap()
            // } else {
            //     depth_rt.texture.render_target_slice_vk_view(
            //         0,
            //         depth_rt.array_slice.unwrap_or(0),
            //         depth_rt.mip_slice.unwrap_or(0),
            //     )
            // };
            image_views.push(image_view);
        };

        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(framebuffer_def.renderpass.vk_renderpass())
            .attachments(&image_views)
            .width(extents.width)
            .height(extents.height)
            .layers(array_length);

        let framebuffer = unsafe {
            device_context
                .device()
                .create_framebuffer(&*framebuffer_create_info, None)?
        };

        let inner = FramebufferVulkanInner {
            device_context: device_context.clone(),
            width: extents.width,
            height: extents.height,
            framebuffer,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}
