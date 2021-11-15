use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanFence;
use crate::{DeviceContextDrc, FenceStatus, GfxResult};
#[cfg(feature = "vulkan")]
use ash::vk;

pub struct Fence {
    device_context: DeviceContextDrc,
    // Set to true when an operation is scheduled to signal this fence
    // Cleared when an operation is scheduled to consume this fence
    submitted: AtomicBool,

    #[cfg(feature = "vulkan")]
    pub(super) platform_fence: VulkanFence,
}

impl Drop for Fence {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_fence.destroy(&self.device_context);
    }
}

impl Fence {
    pub fn new(device_context: &DeviceContextDrc) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_fence = VulkanFence::new(device_context).map_err(|e| {
            log::error!("Error creating platform fence {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        Ok(Self {
            device_context: device_context.clone(),
            submitted: AtomicBool::new(false),
            #[cfg(any(feature = "vulkan"))]
            platform_fence,
        })
    }

    pub fn submitted(&self) -> bool {
        self.submitted.load(Ordering::Relaxed)
    }

    pub fn set_submitted(&self, available: bool) {
        self.submitted.store(available, Ordering::Relaxed);
    }

    pub fn wait(&self) -> GfxResult<()> {
        Self::wait_for_fences(&self.device_context, &[self])
    }

    pub fn wait_for_fences(device_context: &DeviceContextDrc, fences: &[&Self]) -> GfxResult<()> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        {
            let mut fence_list = Vec::with_capacity(fences.len());
            for fence in fences {
                if fence.submitted() {
                    fence_list.push(fence.vk_fence());
                }
            }

            #[cfg(feature = "vulkan")]
            VulkanFence::wait_for_fences(device_context, &fence_list)?;

            for fence in fences {
                fence.set_submitted(false);
            }

            Ok(())
        }
    }

    pub fn get_fence_status(&self) -> GfxResult<FenceStatus> {
        if !self.submitted() {
            Ok(FenceStatus::Unsubmitted)
        } else {
            #[cfg(not(any(feature = "vulkan")))]
            unimplemented!();

            #[cfg(any(feature = "vulkan"))]
            {
                #[cfg(any(feature = "vulkan"))]
                let status = self.platform_fence.get_fence_status(&self.device_context);
                if status.is_ok() && FenceStatus::Complete == status.clone().unwrap() {
                    self.set_submitted(false);
                }
                status
            }
        }
    }

    #[cfg(feature = "vulkan")]
    pub fn vk_fence(&self) -> vk::Fence {
        self.platform_fence.vk_fence()
    }
}
