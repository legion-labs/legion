use ash::vk;

use crate::{DeviceContext, FenceStatus, GfxResult};

pub(crate) struct VulkanFence {
    vk_fence: vk::Fence,
}

impl Drop for VulkanFence {
    fn drop(&mut self) {}
}

impl VulkanFence {
    pub fn new(device_context: &DeviceContext) -> GfxResult<Self> {
        let create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::empty());

        let vk_fence = unsafe {
            device_context
                .inner
                .platform_device_context
                .device()
                .create_fence(&*create_info, None)?
        };

        Ok(Self { vk_fence })
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .inner
                .platform_device_context
                .device()
                .destroy_fence(self.vk_fence, None);
        }
    }

    pub fn vk_fence(&self) -> vk::Fence {
        self.vk_fence
    }

    pub fn wait_for_fences(
        device_context: &DeviceContext,
        fence_list: &[vk::Fence],
    ) -> GfxResult<()> {
        if !fence_list.is_empty() {
            let device = device_context.platform_device();
            unsafe {
                device.wait_for_fences(fence_list, true, std::u64::MAX)?;
                device.reset_fences(fence_list)?;
            }
        }

        Ok(())
    }

    pub fn get_fence_status(&self, device_context: &DeviceContext) -> GfxResult<FenceStatus> {
        let device = device_context.platform_device();
        unsafe {
            let is_ready = device.get_fence_status(self.vk_fence)?;
            if is_ready {
                device.reset_fences(&[self.vk_fence])?;
            }

            if is_ready {
                Ok(FenceStatus::Complete)
            } else {
                Ok(FenceStatus::Incomplete)
            }
        }
    }
}
