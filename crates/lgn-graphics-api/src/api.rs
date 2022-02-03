use std::sync::Arc;

use crate::{
    backends::{BackendApi, BackendInstance},
    ApiDef, DeviceContext, GfxResult,
};

pub struct Instance<'a> {
    pub(crate) backend_instance: &'a BackendInstance,
}

pub struct GfxApi {
    device_context: Option<DeviceContext>,

    // TEMP: "dead code" because it keeps the API alice by being present but is not invoked
    // will be fixed once we call enumerate extensions on it
    #[allow(dead_code)]
    pub(super) backend_api: BackendApi,
}

impl Drop for GfxApi {
    fn drop(&mut self) {
        self.destroy().unwrap();
    }
}

impl GfxApi {
    /// # Safety
    ///
    /// GPU programming is fundamentally unsafe, so all  APIs that interact with
    /// the GPU should be considered unsafe. However,  APIs are only gated
    /// by unsafe if they can cause undefined behavior on the CPU for
    /// reasons other than interacting with the GPU.
    #[allow(unsafe_code)]
    pub unsafe fn new(api_def: &ApiDef) -> GfxResult<Self> {
        let (platform_api, device_context) = BackendApi::new(api_def)?;

        Ok(Self {
            device_context: Some(device_context),
            backend_api: platform_api,
        })
    }

    fn destroy(&mut self) -> GfxResult<()> {
        if let Some(device_context) = self.device_context.take() {
            // Clear any internal caches that may hold references to the device
            BackendApi::destroy(&device_context);
            let inner = device_context.inner.clone();

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            let create_index = device_context.create_index;

            // This should be the final device context
            std::mem::drop(device_context);

            let strong_count = Arc::strong_count(&inner);
            match Arc::try_unwrap(inner) {
                Ok(inner) => std::mem::drop(inner),
                Err(_arc) => {
                    #[cfg(debug_assertions)]
                    #[cfg(feature = "track-device-contexts")]
                    {
                        #[allow(clippy::used_underscore_binding)]
                        let mut all_contexts = _arc.all_contexts.lock().unwrap();
                        all_contexts.remove(&create_index);
                        for (k, v) in all_contexts.iter_mut() {
                            v.resolve();
                            println!("context allocation: {}\n{:?}", k, v);
                        }
                    }

                    return Err(format!(
                        "Could not destroy device, {} references to it exist",
                        strong_count
                    )
                    .into());
                }
            }
        }

        Ok(())
    }

    pub fn device_context(&self) -> &DeviceContext {
        self.device_context.as_ref().unwrap()
    }
}
