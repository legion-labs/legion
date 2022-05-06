use std::sync::atomic::{AtomicBool, Ordering};

use crate::{backends::BackendFence, DeviceContext, FenceStatus, GfxResult};

pub struct Fence {
    pub(crate) device_context: DeviceContext,
    // Set to true when an operation is scheduled to signal this fence
    // Cleared when an operation is scheduled to consume this fence
    submitted: AtomicBool,
    pub(crate) backend_fence: BackendFence,
}

impl Drop for Fence {
    fn drop(&mut self) {
        self.backend_fence.destroy(&self.device_context);
    }
}

impl Fence {
    pub fn new(device_context: &DeviceContext) -> Self {
        let backend_fence = BackendFence::new(device_context);

        Self {
            device_context: device_context.clone(),
            submitted: AtomicBool::new(false),
            backend_fence,
        }
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

    pub fn wait_for_fences(device_context: &DeviceContext, fences: &[&Self]) -> GfxResult<()> {
        let mut fence_list = Vec::with_capacity(fences.len());
        for fence in fences {
            if fence.submitted() {
                fence_list.push(*fence);
            }
        }

        if !fence_list.is_empty() {
            Self::backend_wait_for_fences(device_context, fence_list.as_slice())?;
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
            let status = self.get_fence_status_platform();
            if status.is_ok() && FenceStatus::Complete == status.clone().unwrap() {
                self.set_submitted(false);
            }
            status
        }
    }
}
