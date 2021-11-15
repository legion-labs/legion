use raw_window_handle::HasRawWindowHandle;

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanSwapchain;
use crate::{DeviceContextDrc, Fence, Format, GfxResult, Semaphore, SwapchainDef, SwapchainImage};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Swapchain {
    device_context: DeviceContextDrc,
    swapchain_def: SwapchainDef,

    #[cfg(feature = "vulkan")]
    platform_swap_chain: VulkanSwapchain,
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_swap_chain.destroy();
    }
}

impl Swapchain {
    pub fn new(
        device_context: &DeviceContextDrc,
        raw_window_handle: &dyn HasRawWindowHandle,
        swapchain_def: &SwapchainDef,
    ) -> GfxResult<Self> {
        //TODO: Check image count of swapchain and update swapchain_def with swapchain.swapchain_images.len();
        let swapchain_def = swapchain_def.clone();

        #[cfg(feature = "vulkan")]
        let platform_swap_chain =
            VulkanSwapchain::new(device_context, raw_window_handle, &swapchain_def)?;

        Ok(Self {
            device_context: device_context.clone(),
            swapchain_def,
            #[cfg(any(feature = "vulkan"))]
            platform_swap_chain,
        })
    }

    pub fn swapchain_def(&self) -> &SwapchainDef {
        &self.swapchain_def
    }

    pub fn image_count(&self) -> usize {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_swap_chain.image_count()
    }

    pub fn format(&self) -> Format {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_swap_chain.format()
    }

    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn platform_swap_chain(&self) -> &VulkanSwapchain {
        &self.platform_swap_chain
    }

    //TODO: Return something like PresentResult?
    pub fn acquire_next_image_fence(&mut self, fence: &Fence) -> GfxResult<SwapchainImage> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_swap_chain.acquire_next_image_fence(fence)
    }

    //TODO: Return something like PresentResult?
    pub fn acquire_next_image_semaphore(
        &mut self,
        semaphore: &Semaphore,
    ) -> GfxResult<SwapchainImage> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_swap_chain
            .acquire_next_image_semaphore(semaphore)
    }

    pub fn rebuild(&mut self, swapchain_def: &SwapchainDef) -> GfxResult<()> {
        self.swapchain_def = swapchain_def.clone();
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_swap_chain
            .rebuild(&self.device_context, swapchain_def)
    }
}
