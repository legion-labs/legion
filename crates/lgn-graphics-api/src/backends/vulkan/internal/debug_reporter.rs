use std::ffi::{CStr, CString};
use std::os::raw::c_void;

use ash::vk;
use ash::{extensions::ext::DebugUtils, vk::Handle};
use lgn_tracing::{debug, error, info, trace, warn};

use crate::backends::vulkan::VulkanDeviceContext;
use crate::{Buffer, CommandBuffer, Texture};

const ERRORS_TO_IGNORE: [&str; 0] = [
    // Temporary - I suspect locally built validation on M1 mac has a bug
    //"VUID-VkWriteDescriptorSet-descriptorType-00332",
    //"VUID-VkWriteDescriptorSet-descriptorType-00333",
    // windows/5700xt can return 0 max surface size when window is resized to (0, 0). Spec
    // states swapchain size must be > 0
    //"VUID-VkSwapchainCreateInfoKHR-imageExtent-01274",
];

/// Callback for vulkan validation layer logging
pub extern "system" fn vulkan_debug_callback(
    flags: vk::DebugUtilsMessageSeverityFlagsEXT,
    _: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> u32 {
    if !data.is_null() {
        let data_ptr = unsafe { &(*data) };
        if !data_ptr.p_message.is_null() {
            let msg_ptr = (*data_ptr).p_message;
            let msg = unsafe { CStr::from_ptr(msg_ptr) };
            if flags.intersects(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
                let mut ignored = false;
                for ignored_error in &ERRORS_TO_IGNORE {
                    if msg.to_string_lossy().contains(ignored_error) {
                        ignored = true;
                        break;
                    }
                }

                if !ignored {
                    error!("{:?}", msg);
                    panic!();
                }
            } else if flags.intersects(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
                warn!("{:?}", msg);
            } else if flags.intersects(vk::DebugUtilsMessageSeverityFlagsEXT::INFO) {
                info!("{:?}", msg);
            } else {
                debug!("{:?}", msg);
            }
        } else {
            error!("Received null message pointer in vulkan_debug_callback");
        }
    } else {
        error!("Received null data pointer in vulkan_debug_callback");
    }

    vk::FALSE
}

/// Handles dropping vulkan debug reporting
pub struct VkDebugReporter {
    pub debug_report_loader: DebugUtils,
    pub debug_callback: vk::DebugUtilsMessengerEXT,
}

impl Drop for VkDebugReporter {
    fn drop(&mut self) {
        unsafe {
            trace!("destroying VkDebugReporter");
            self.debug_report_loader
                .destroy_debug_utils_messenger(self.debug_callback, None);
            trace!("destroyed VkDebugReporter");
        }
    }
}

impl VkDebugReporter {
    pub(crate) fn set_texture_name(
        &self,
        device_context: &VulkanDeviceContext,
        texture: &Texture,
        name: &str,
    ) {
        let name = &CString::new(name).unwrap();
        let object_name_info = ash::vk::DebugUtilsObjectNameInfoEXT::builder()
            .object_type(ash::vk::ObjectType::IMAGE)
            .object_handle(texture.vk_image().as_raw())
            .object_name(name)
            .build();

        unsafe {
            self.debug_report_loader
                .debug_utils_set_object_name(device_context.vk_device().handle(), &object_name_info)
                .unwrap();
        }
    }

    pub(crate) fn set_buffer_name(
        &self,
        device_context: &VulkanDeviceContext,
        buffer: &Buffer,
        name: &str,
    ) {
        let name = &CString::new(name).unwrap();
        let object_name_info = ash::vk::DebugUtilsObjectNameInfoEXT::builder()
            .object_type(ash::vk::ObjectType::BUFFER)
            .object_handle(buffer.vk_buffer().as_raw())
            .object_name(name)
            .build();

        unsafe {
            self.debug_report_loader
                .debug_utils_set_object_name(device_context.vk_device().handle(), &object_name_info)
                .unwrap();
        }
    }

    pub(crate) fn begin_label(
        &self,
        command_buffer: &CommandBuffer,
        label: &str, /*, todo: optional color? */
    ) {
        let label = CString::new(label).unwrap();
        let label = ash::vk::DebugUtilsLabelEXT::builder()
            .label_name(&label)
            .build();

        unsafe {
            self.debug_report_loader
                .cmd_begin_debug_utils_label(command_buffer.vk_command_buffer(), &label);
        }
    }

    pub(crate) fn end_label(&self, command_buffer: &CommandBuffer) {
        unsafe {
            self.debug_report_loader
                .cmd_end_debug_utils_label(command_buffer.vk_command_buffer());
        }
    }
}
