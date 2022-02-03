use raw_window_handle::HasRawWindowHandle;

use crate::{
    backends::BackendSwapchain, DeviceContext, Fence, Format, GfxResult, Semaphore, SwapchainImage,
};

/// Used to create a `Swapchain`
#[derive(Clone)]
pub struct SwapchainDef {
    pub width: u32,
    pub height: u32,
    pub enable_vsync: bool,
    // image count?
}

pub struct Swapchain {
    pub(crate) device_context: DeviceContext,
    pub(crate) swapchain_def: SwapchainDef,
    pub(crate) backend_swapchain: BackendSwapchain,
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        self.backend_swapchain.destroy();
    }
}

impl Swapchain {
    pub fn new(
        device_context: &DeviceContext,
        raw_window_handle: &dyn HasRawWindowHandle,
        swapchain_def: &SwapchainDef,
    ) -> GfxResult<Self> {
        //TODO: Check image count of swapchain and update swapchain_def with
        // swapchain.swapchain_images.len();
        let swapchain_def = swapchain_def.clone();

        let backend_swapchain =
            BackendSwapchain::new(device_context, raw_window_handle, &swapchain_def)?;

        Ok(Self {
            device_context: device_context.clone(),
            swapchain_def,
            backend_swapchain,
        })
    }

    pub fn swapchain_def(&self) -> &SwapchainDef {
        &self.swapchain_def
    }

    pub fn image_count(&self) -> usize {
        self.backend_image_count()
    }

    pub fn format(&self) -> Format {
        self.backend_format()
    }

    //TODO: Return something like PresentResult?
    pub fn acquire_next_image_fence(&mut self, fence: &Fence) -> GfxResult<SwapchainImage> {
        self.backend_acquire_next_image_fence(fence)
    }

    //TODO: Return something like PresentResult?
    pub fn acquire_next_image_semaphore(
        &mut self,
        semaphore: &Semaphore,
    ) -> GfxResult<SwapchainImage> {
        self.backend_acquire_next_image_semaphore(semaphore)
    }

    pub fn rebuild(&mut self, swapchain_def: &SwapchainDef) -> GfxResult<()> {
        self.backend_rebuild(swapchain_def)
    }
}
