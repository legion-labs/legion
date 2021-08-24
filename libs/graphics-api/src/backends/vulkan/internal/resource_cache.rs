use crate::backends::vulkan::{VulkanFramebufferCache, VulkanRenderpassCache};
use std::sync::Mutex;

pub(crate) struct DeviceVulkanResourceCacheInner {
    pub(crate) renderpass_cache: VulkanRenderpassCache,
    pub(crate) framebuffer_cache: VulkanFramebufferCache,
}

pub(crate) struct DeviceVulkanResourceCache {
    pub(crate) inner: Mutex<DeviceVulkanResourceCacheInner>,
}

impl DeviceVulkanResourceCache {
    pub(crate) fn clear_caches(&self) {
        let mut lock = self.inner.lock().unwrap();
        lock.framebuffer_cache.clear();
        lock.renderpass_cache.clear();
    }
}

impl Default for DeviceVulkanResourceCache {
    fn default() -> Self {
        let inner = DeviceVulkanResourceCacheInner {
            renderpass_cache: VulkanRenderpassCache::new(200),
            framebuffer_cache: VulkanFramebufferCache::new(200),
        };

        Self {
            inner: Mutex::new(inner),
        }
    }
}
