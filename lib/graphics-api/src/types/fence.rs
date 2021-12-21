use std::sync::atomic::{AtomicBool, Ordering};

use lgn_telemetry::error;

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanFence;
use crate::{DeviceContext, FenceStatus, GfxResult};

pub(crate) struct FenceInner {
    pub(crate) device_context: DeviceContext,
    // Set to true when an operation is scheduled to signal this fence
    // Cleared when an operation is scheduled to consume this fence
    submitted: AtomicBool,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_fence: VulkanFence,
}

pub struct Fence {
    pub(crate) inner: Box<FenceInner>,
}

impl Drop for Fence {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.inner
            .platform_fence
            .destroy(&self.inner.device_context);
    }
}

impl Fence {
    pub fn new(device_context: &DeviceContext) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_fence = VulkanFence::new(device_context).map_err(|e| {
            error!("Error creating platform fence {:?}", e);
            ash::vk::Result::ERROR_UNKNOWN
        })?;

        Ok(Self {
            inner: Box::new(FenceInner {
                device_context: device_context.clone(),
                submitted: AtomicBool::new(false),
                #[cfg(any(feature = "vulkan"))]
                platform_fence,
            }),
        })
    }

    pub fn submitted(&self) -> bool {
        self.inner.submitted.load(Ordering::Relaxed)
    }

    pub fn set_submitted(&self, available: bool) {
        self.inner.submitted.store(available, Ordering::Relaxed);
    }

    pub fn wait(&self) -> GfxResult<()> {
        Self::wait_for_fences(&self.inner.device_context, &[self])
    }

    pub fn wait_for_fences(device_context: &DeviceContext, fences: &[&Self]) -> GfxResult<()> {
        let mut fence_list = Vec::with_capacity(fences.len());
        for fence in fences {
            if fence.submitted() {
                fence_list.push(*fence);
            }
        }

        if !fence_list.is_empty() {
            #[cfg(any(feature = "vulkan"))]
            Self::wait_for_fences_platform(device_context, fence_list.as_slice())?;
        }

        for fence in fences {
            fence.set_submitted(false);
        }

        Ok(())
    }

    pub fn get_fence_status(&self) -> GfxResult<FenceStatus> {
        if !self.submitted() {
            Ok(FenceStatus::Unsubmitted)
        } else {
            #[cfg(not(any(feature = "vulkan")))]
            unimplemented!();

            #[cfg(any(feature = "vulkan"))]
            {
                let status = self.get_fence_status_platform();
                if status.is_ok() && FenceStatus::Complete == status.clone().unwrap() {
                    self.set_submitted(false);
                }
                status
            }
        }
    }
}
