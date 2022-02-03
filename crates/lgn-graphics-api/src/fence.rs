use std::sync::atomic::{AtomicBool, Ordering};

use crate::{backends::BackendFence, DeviceContext, FenceStatus, GfxResult};

pub(crate) struct FenceInner {
    pub(crate) device_context: DeviceContext,
    // Set to true when an operation is scheduled to signal this fence
    // Cleared when an operation is scheduled to consume this fence
    submitted: AtomicBool,
    pub(crate) backend_fence: BackendFence,
}

pub struct Fence {
    pub(crate) inner: Box<FenceInner>,
}

impl Drop for Fence {
    fn drop(&mut self) {
        self.inner.backend_fence.destroy(&self.inner.device_context);
    }
}

impl Fence {
    pub fn new(device_context: &DeviceContext) -> GfxResult<Self> {
        let backend_fence = BackendFence::new(device_context)?;

        Ok(Self {
            inner: Box::new(FenceInner {
                device_context: device_context.clone(),
                submitted: AtomicBool::new(false),
                backend_fence,
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
