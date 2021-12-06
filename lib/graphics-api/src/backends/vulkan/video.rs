#![allow(dead_code)]

use std::ffi::CStr;

use ash::vk;

use crate::{DeviceContext, GfxResult};

#[derive(Clone)]
pub struct VideoQueue {
    device_ctx: DeviceContext,
    video_queue_fn: vk::KhrVideoQueueFn,
}

//enum StdVideoH264ProfileIdc {
//    std_video_h264_profile_idc_baseline             = 66, /* Only constrained baseline is supported */
//    std_video_h264_profile_idc_main                 = 77,
//    std_video_h264_profile_idc_high                 = 100,
//    std_video_h264_profile_idc_high_444_predictive  = 244,
//    std_video_h264_profile_idc_invalid              = 0x7FFFFFFF
//} StdVideoH264ProfileIdc;

impl VideoQueue {
    pub(crate) fn new(device_ctx: DeviceContext) -> Self {
        let mut video_queue_fn = vk::KhrVideoQueueFn::load(|name| unsafe {
            std::mem::transmute(
                device_ctx
                    .vk_instance()
                    .get_device_proc_addr(device_ctx.vk_device().handle(), name.as_ptr()),
            )
        });
        // The following functions need to be loaded from the instance rather than from the device
        {
            let func = unsafe {
                let cname = ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                    b"vkGetPhysicalDeviceVideoCapabilitiesKHR\0",
                );
                device_ctx
                    .vk_entry()
                    .get_instance_proc_addr(device_ctx.vk_instance().handle(), cname.as_ptr())
            };
            if let Some(func) = func {
                video_queue_fn.get_physical_device_video_capabilities_khr =
                    unsafe { std::mem::transmute(func) };
            }
        }
        {
            let func = unsafe {
                let cname = ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                    b"vkGetPhysicalDeviceVideoFormatPropertiesKHR\0",
                );
                device_ctx
                    .vk_entry()
                    .get_instance_proc_addr(device_ctx.vk_instance().handle(), cname.as_ptr())
            };
            if let Some(func) = func {
                video_queue_fn.get_physical_device_video_format_properties_khr =
                    unsafe { std::mem::transmute(func) };
            }
        }
        Self {
            device_ctx,
            video_queue_fn,
        }
    }

    pub(crate) fn name() -> &'static CStr {
        vk::KhrVideoQueueFn::name()
    }

    pub(crate) fn fp(&self) -> &vk::KhrVideoQueueFn {
        &self.video_queue_fn
    }

    pub(crate) fn get_physical_device_video_capabilities(
        &self,
        video_profile: &vk::VideoProfileKHR,
        capabilities: &mut vk::VideoCapabilitiesKHR,
    ) -> GfxResult<()> {
        unsafe {
            self.video_queue_fn
                .get_physical_device_video_capabilities_khr(
                    self.device_ctx.vk_physical_device(),
                    video_profile,
                    capabilities,
                )
                .result()?;
        }
        Ok(())
    }

    pub(crate) fn get_physical_device_video_format_properties(
        &self,
        video_format_info: &vk::PhysicalDeviceVideoFormatInfoKHR,
        video_format_properties: &mut Vec<vk::VideoFormatPropertiesKHR>,
    ) -> GfxResult<()> {
        let mut video_format_info_count = 0;
        unsafe {
            self.video_queue_fn
                .get_physical_device_video_format_properties_khr(
                    self.device_ctx.vk_physical_device(),
                    video_format_info,
                    &mut video_format_info_count,
                    std::ptr::null_mut(),
                )
                .result()?;
            video_format_properties.resize(
                video_format_info_count as _,
                vk::VideoFormatPropertiesKHR::default(),
            );
            self.video_queue_fn
                .get_physical_device_video_format_properties_khr(
                    self.device_ctx.vk_physical_device(),
                    video_format_info,
                    &mut video_format_info_count,
                    video_format_properties.as_mut_ptr(),
                )
                .result()?;
        }
        Ok(())
    }

    pub(crate) fn create_video_session(
        &self,
        video_session_info: &vk::VideoSessionCreateInfoKHR,
    ) -> GfxResult<vk::VideoSessionKHR> {
        let mut video_session = vk::VideoSessionKHR::default();
        unsafe {
            self.video_queue_fn
                .create_video_session_khr(
                    self.device_ctx.vk_device().handle(),
                    video_session_info,
                    std::ptr::null(),
                    &mut video_session,
                )
                .result()?;
        }
        Ok(video_session)
    }

    pub(crate) fn destroy_video_session(&self, video_session: vk::VideoSessionKHR) {
        unsafe {
            self.video_queue_fn.destroy_video_session_khr(
                self.device_ctx.vk_device().handle(),
                video_session,
                std::ptr::null(),
            );
        }
    }

    pub(crate) fn get_video_session_memory_requirements(
        &self,
        video_session: vk::VideoSessionKHR,
        memory_requirements: &mut Vec<vk::MemoryRequirements2>,
        video_session_memory_requirements: &mut Vec<vk::VideoGetMemoryPropertiesKHR>,
    ) -> GfxResult<()> {
        let mut video_session_memory_requirements_count = 0;
        unsafe {
            self.video_queue_fn
                .get_video_session_memory_requirements_khr(
                    self.device_ctx.vk_device().handle(),
                    video_session,
                    &mut video_session_memory_requirements_count,
                    std::ptr::null_mut(),
                )
                .result()?;
            memory_requirements.resize(
                video_session_memory_requirements_count as usize,
                vk::MemoryRequirements2::default(),
            );
            video_session_memory_requirements.resize(
                video_session_memory_requirements_count as usize,
                vk::VideoGetMemoryPropertiesKHR::default(),
            );
            for i in 0..video_session_memory_requirements_count {
                video_session_memory_requirements[i as usize].p_memory_requirements =
                    &mut memory_requirements[i as usize];
            }

            self.video_queue_fn
                .get_video_session_memory_requirements_khr(
                    self.device_ctx.vk_device().handle(),
                    video_session,
                    &mut video_session_memory_requirements_count,
                    video_session_memory_requirements.as_mut_ptr(),
                )
                .result()?;

            for i in 0..video_session_memory_requirements_count {
                video_session_memory_requirements[i as usize].p_memory_requirements =
                    std::ptr::null_mut();
            }
        }

        Ok(())
    }
}
