use std::ffi::CString;
use std::fmt;

use super::internal::VkInstance;
use crate::{ApiDef, DeviceContext, GfxResult, Instance};

pub(crate) struct VulkanApi {
    instance: VkInstance,
}

impl fmt::Debug for VulkanApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VulkanApi")
            .field("instance", &self.instance.instance.handle())
            .finish()
    }
}

impl VulkanApi {
    /// # Safety
    ///
    /// GPU programming is fundamentally unsafe, so all  APIs that interact with the GPU should
    /// be considered unsafe. However,  APIs are only gated by unsafe if they can cause undefined
    /// behavior on the CPU for reasons other than interacting with the GPU.
    pub unsafe fn new(api_def: &ApiDef) -> GfxResult<(Self, DeviceContext)> {
        let app_name = CString::new(api_def.app_name.clone())
            .expect("app name should not contain a byte set to 0");
        let entry = ash::Entry::new()?;
        let vk_instance = VkInstance::new(
            entry,
            &app_name,
            api_def.validation_mode,
            api_def.windowing_mode,
        )?;

        let device_context = DeviceContext::new(
            &Instance {
                platform_instance: &vk_instance,
            },
            api_def,
        )?;

        Ok((
            Self {
                instance: vk_instance,
            },
            device_context,
        ))
    }

    pub fn destroy(device_context: &DeviceContext) {
        device_context.resource_cache().clear_caches();
        device_context.deferred_dropper().destroy();
    }
}
