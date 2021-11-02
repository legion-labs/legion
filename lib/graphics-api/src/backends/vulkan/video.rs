use std::ffi::CStr;

use ash::vk;

//use crate::GfxResult;
use super::VulkanDeviceContext;

#[derive(Clone)]
pub struct VideoQueue {
    device_ctx: VulkanDeviceContext,
    video_queue_fn: vk::KhrVideoQueueFn,
}

impl VideoQueue {
    pub fn new(device_ctx: VulkanDeviceContext) -> Self {
        let video_queue_fn = vk::KhrVideoQueueFn::load(|name| unsafe {
            std::mem::transmute(
                device_ctx
                    .instance()
                    .get_device_proc_addr(device_ctx.device().handle(), name.as_ptr()),
            )
        });
        Self {
            device_ctx,
            video_queue_fn,
        }
    }

    pub fn name() -> &'static CStr {
        vk::KhrVideoQueueFn::name()
    }

    pub fn fp(&self) -> &vk::KhrVideoQueueFn {
        &self.video_queue_fn
    }

    //pub fn get_physical_device_video_capabilities_khr(
    //    &self,
    //    //physical_device: PhysicalDevice,
    //    video_profile: &vk::VideoProfileKHR,
    //    capabilities: &mut vk::VideoCapabilitiesKHR,
    //) -> GfxResult<()> {
    //    //unsafe {
    //    //    self.video_queue_fn
    //    //        .get_physical_device_video_capabilities_khr(
    //    //            self.device_ctx.physical_device(),
    //    //            video_profile,
    //    //            capabilities,
    //    //        )
    //    //        .result().map_err(|e| GfxError::VkError(r))
    //    //}
    //    Ok(())
    //}
}
