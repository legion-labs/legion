use std::sync::Arc;

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanApi;
#[cfg(not(any(feature = "vulkan")))]
use crate::Instance;
use crate::{ApiDef, DeviceContext, GfxResult};

pub struct GfxApi {
    device_context: Option<DeviceContext>,

    // TEMP: "dead code" because it keeps the API alice by being present but is not invoked
    // will be fixed once we call enumerate extensions on it
    #[allow(dead_code)]
    #[cfg(feature = "vulkan")]
    pub(super) platform_api: VulkanApi,
}

impl Drop for GfxApi {
    fn drop(&mut self) {
        self.destroy().unwrap();
    }
}

impl GfxApi {
    /// # Safety
    ///
    /// GPU programming is fundamentally unsafe, so all  APIs that interact with the GPU should
    /// be considered unsafe. However,  APIs are only gated by unsafe if they can cause undefined
    /// behavior on the CPU for reasons other than interacting with the GPU.
    #[allow(unsafe_code)]
    pub unsafe fn new(api_def: &ApiDef) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let (platform_api, device_context) = VulkanApi::new(api_def).map_err(|e| {
            log::error!("Error creating buffer {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        #[cfg(not(any(feature = "vulkan")))]
        let device_context = DeviceContext::new(&Instance {}, api_def).unwrap();

        Ok(Self {
            device_context: Some(device_context),
            #[cfg(any(feature = "vulkan"))]
            platform_api,
        })
    }

    fn destroy(&mut self) -> GfxResult<()> {
        if let Some(device_context) = self.device_context.take() {
            // Clear any internal caches that may hold references to the device
            #[cfg(feature = "vulkan")]
            VulkanApi::destroy(&device_context);
            let inner = device_context.inner.clone();

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            let _create_index = device_context.create_index;

            // This should be the final device context
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

    pub fn device_context(&self) -> &DeviceContext {
        self.device_context.as_ref().unwrap()
    }
}
