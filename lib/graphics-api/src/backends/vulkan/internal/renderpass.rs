use crate::backends::vulkan::VulkanDeviceContext;
use crate::{Format, GfxResult, LoadOp, StoreOp};
use ash::vk;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct VulkanRenderpassColorAttachment {
    pub(crate) format: Format,
    pub(crate) load_op: LoadOp,
    pub(crate) store_op: StoreOp,
}

#[derive(Clone, Debug)]
pub(crate) struct RenderpassVulkanResolveAttachment {
    pub(crate) format: Format,
}

#[derive(Clone, Debug)]
pub(crate) struct VulkanRenderpassDepthAttachment {
    pub(crate) format: Format,
    pub(crate) depth_load_op: LoadOp,
    pub(crate) stencil_load_op: LoadOp,
    pub(crate) depth_store_op: StoreOp,
    pub(crate) stencil_store_op: StoreOp,
}

#[derive(Clone, Debug)]
pub(crate) struct VulkanRenderpassDef {
    pub(crate) color_attachments: Vec<VulkanRenderpassColorAttachment>,
    pub(crate) depth_attachment: Option<VulkanRenderpassDepthAttachment>,
}

pub(crate) struct RenderpassVulkanInner {
    device_context: VulkanDeviceContext,
    renderpass: vk::RenderPass,
}

impl Drop for RenderpassVulkanInner {
    fn drop(&mut self) {
        unsafe {
            self.device_context
                .device()
                .destroy_render_pass(self.renderpass, None);
        }
    }
}

#[derive(Clone)]
pub(crate) struct VulkanRenderpass {
    inner: Arc<RenderpassVulkanInner>,
}

impl VulkanRenderpass {
    pub fn vk_renderpass(&self) -> vk::RenderPass {
        self.inner.renderpass
    }

    pub fn new(
        device_context: &VulkanDeviceContext,
        renderpass_def: &VulkanRenderpassDef,
    ) -> GfxResult<Self> {
        let mut attachments = Vec::with_capacity(renderpass_def.color_attachments.len() + 1);
        let mut color_attachment_refs = Vec::with_capacity(renderpass_def.color_attachments.len());

        for (color_attachment_index, color_attachment) in
            renderpass_def.color_attachments.iter().enumerate()
        {
            attachments.push(
                vk::AttachmentDescription::builder()
                    .format(color_attachment.format.into())
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(color_attachment.load_op.into())
                    .store_op(color_attachment.store_op.into())
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .build(),
            );

            color_attachment_refs.push(
                vk::AttachmentReference::builder()
                    .attachment(color_attachment_index as u32)
                    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .build(),
            );
        }

        let mut depth_stencil_attachment_ref = None;
        if let Some(depth_attachment) = &renderpass_def.depth_attachment {
            assert_ne!(depth_attachment.format, Format::UNDEFINED);
            let attachment_index = attachments.len() as u32;
            attachments.push(
                vk::AttachmentDescription::builder()
                    .format(depth_attachment.format.into())
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(depth_attachment.depth_load_op.into())
                    .store_op(depth_attachment.depth_store_op.into())
                    .stencil_load_op(depth_attachment.stencil_load_op.into())
                    .stencil_store_op(depth_attachment.stencil_store_op.into())
                    .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .build(),
            );

            depth_stencil_attachment_ref = Some(
                vk::AttachmentReference::builder()
                    .attachment(attachment_index)
                    .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .build(),
            );
        }

        let mut subpass_description = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);

        if let Some(depth_stencil_attachment_ref) = depth_stencil_attachment_ref.as_ref() {
            subpass_description =
                subpass_description.depth_stencil_attachment(depth_stencil_attachment_ref);
        }

        let subpass_descriptions = [subpass_description.build()];

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpass_descriptions);

        let renderpass = unsafe {
            device_context
                .device()
                .create_render_pass(&*renderpass_create_info, None)?
        };

        let inner = RenderpassVulkanInner {
            device_context: device_context.clone(),
            renderpass,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}
