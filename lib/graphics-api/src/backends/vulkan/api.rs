use std::ffi::CString;
use std::{fmt, sync::Arc};

use super::internal::VkInstance;
use crate::{ApiDef, DeviceContextDrc, GfxResult, Instance};

pub struct VulkanApi {
    instance: VkInstance,
    device_context: Option<DeviceContextDrc>,
}

impl Drop for VulkanApi {
    fn drop(&mut self) {
        self.destroy().unwrap();
    }
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
    pub unsafe fn new(api_def: &ApiDef) -> GfxResult<Self> {
        let app_name = CString::new(api_def.app_name.clone())
            .expect("app name should not contain a byte set to 0");
        let entry = ash::Entry::new()?;
        let vk_instance = VkInstance::new(
            entry,
            &app_name,
            api_def.validation_mode,
            api_def.windowing_mode,
        )?;

        let device_context = Some(DeviceContextDrc::new(
            &Instance {
                platform_instance: &vk_instance,
            },
            api_def,
        )?);

        Ok(Self {
            instance: vk_instance,
            device_context,
        })
    }

    fn destroy(&mut self) -> GfxResult<()> {
        if let Some(device_context) = self.device_context.take() {
            // Clear any internal caches that may hold references to the device
            let inner = device_context.inner.clone();
            inner.platform_device_context.resource_cache.clear_caches();
            inner.deferred_dropper.destroy();

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            let _create_index = device_context.create_index;

            // Thsi should be the final device context
            std::mem::drop(device_context);

            let strong_count = Arc::strong_count(&inner);
            match Arc::try_unwrap(inner) {
                Ok(inner) => std::mem::drop(inner),
                Err(_arc) => {
                    return Err(format!(
                        "Could not destroy device, {} references to it exist",
                        strong_count
                    )
                    .into());

                    #[cfg(debug_assertions)]
                    #[cfg(feature = "track-device-contexts")]
                    {
                        let mut all_contexts = _arc.all_contexts.lock().unwrap();
                        all_contexts.remove(&_create_index);
                        for (k, v) in all_contexts.iter_mut() {
                            v.resolve();
                            println!("context allocation: {}\n{:?}", k, v);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn device_context(&self) -> &DeviceContextDrc {
        self.device_context.as_ref().unwrap()
    }
}
