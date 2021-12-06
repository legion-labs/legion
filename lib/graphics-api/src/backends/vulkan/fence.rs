use ash::vk;

use crate::{DeviceContext, Fence, FenceStatus, GfxResult};

pub(crate) struct VulkanFence {
    vk_fence: vk::Fence,
}

impl VulkanFence {
    pub(crate) fn new(device_context: &DeviceContext) -> GfxResult<Self> {
        let create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::empty());

        let vk_fence = unsafe {
            device_context
                .vk_device()
                .create_fence(&*create_info, None)?
        };

        Ok(Self { vk_fence })
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .destroy_fence(self.vk_fence, None);
        }
    }
}

impl Fence {
    pub(crate) fn vk_fence(&self) -> vk::Fence {
        self.inner.platform_fence.vk_fence
    }

    pub(crate) fn wait_for_fences_platform(
        device_context: &DeviceContext,
        fence_list: &[&Self],
    ) -> GfxResult<()> {
        assert!(!fence_list.is_empty());

        let vk_fence_list: Vec<ash::vk::Fence> = fence_list.iter().map(|f| f.vk_fence()).collect();

        let device = device_context.vk_device();
        unsafe {
            device.wait_for_fences(&vk_fence_list, true, std::u64::MAX)?;
            device.reset_fences(&vk_fence_list)?;
        }

        Ok(())
    }

    pub(crate) fn get_fence_status_platform(&self) -> GfxResult<FenceStatus> {
        let device = self.inner.device_context.vk_device();
        unsafe {
            let is_ready = device.get_fence_status(self.inner.platform_fence.vk_fence)?;
            if is_ready {
                device.reset_fences(&[self.inner.platform_fence.vk_fence])?;
            }

            if is_ready {
                Ok(FenceStatus::Complete)
            } else {
                Ok(FenceStatus::Incomplete)
            }
        }
    }
}
