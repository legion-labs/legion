use super::{
    internal::*, VulkanBuffer, VulkanCommandBuffer, VulkanCommandPool, VulkanDescriptorSetArray,
    VulkanDescriptorSetHandle, VulkanFence, VulkanPipeline, VulkanQueue, VulkanRootSignature,
    VulkanSampler, VulkanSemaphore, VulkanShader, VulkanShaderModule, VulkanSwapchain,
    VulkanTexture, VulkanDescriptorSetLayout
};
use ash::vk;
use raw_window_handle::HasRawWindowHandle;
use std::{fmt, sync::Arc};

use super::{VulkanDeviceContext, VulkanDeviceContextInner};
use crate::*;
use std::ffi::CString;

/// Vulkan-specific configuration
pub struct ApiDefVulkan {
    /// Used as a hint for drivers for what is being run. There are no special requirements for
    /// this. It is not visible to end-users.
    pub app_name: CString,

    /// Used to enable/disable validation at runtime. Not all APIs allow this. Validation is helpful
    /// during development but very expensive. Applications should not ship with validation enabled.
    pub validation_mode: ValidationMode,
    // The OS-specific layers/extensions are already included. Debug layers/extension are included
    // if enable_validation is true
    //TODO: Additional instance layer names
    //TODO: Additional instance extension names
    //TODO: Additional device extension names
}

impl Default for ApiDefVulkan {
    fn default() -> Self {
        Self {
            app_name: CString::new(" Application").unwrap(),
            validation_mode: Default::default(),
        }
    }
}

pub struct VulkanApi {
    instance: VkInstance,
    device_context: Option<VulkanDeviceContext>,
}

impl Drop for VulkanApi {
    fn drop(&mut self) {
        self.destroy().unwrap();
    }
}

impl fmt::Debug for VulkanApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VulkanApi")
            .field("instance", &self.instance.instance.handle())
            .finish()
    }
}

impl GfxApi for VulkanApi {
    fn device_context(&self) -> &VulkanDeviceContext {
        self.device_context.as_ref().unwrap()
    }

    fn destroy(&mut self) -> GfxResult<()> {
        if let Some(device_context) = self.device_context.take() {
            // Clear any internal caches that may hold references to the device
            let inner = device_context.inner.clone();
            inner.descriptor_heap.clear_pools(device_context.device());
            inner.resource_cache.clear_caches();

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            let _create_index = device_context.create_index;

            // Thsi should be the final device context
            std::mem::drop(device_context);

            let _strong_count = Arc::strong_count(&inner);
            match Arc::try_unwrap(inner) {
                Ok(inner) => std::mem::drop(inner),
                Err(_arc) => {
                    return Err(format!(
                        "Could not destroy device, {} references to it exist",
                        _strong_count
                    )
                    .into());

                    #[cfg(debug_assertions)]
                    #[cfg(feature = "track-device-contexts")]
                    {
                        let mut all_contexts = _arc.all_contexts.lock().unwrap();
                        all_contexts.remove(&_create_index);
                        for (k, v) in all_contexts.iter_mut() {
                            v.resolve();
                            println!("context allocation: {}\n{:?}", k, v);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    type DeviceContext = VulkanDeviceContext;
    type Buffer = VulkanBuffer;
    type Texture = VulkanTexture;
    type Sampler = VulkanSampler;
    type ShaderModule = VulkanShaderModule;
    type Shader = VulkanShader;
    type DescriptorSetLayout = VulkanDescriptorSetLayout;
    type RootSignature = VulkanRootSignature;
    type Pipeline = VulkanPipeline;
    type DescriptorSetHandle = VulkanDescriptorSetHandle;
    type DescriptorSetArray = VulkanDescriptorSetArray;
    type Queue = VulkanQueue;
    type CommandPool = VulkanCommandPool;
    type CommandBuffer = VulkanCommandBuffer;
    type Fence = VulkanFence;
    type Semaphore = VulkanSemaphore;
    type Swapchain = VulkanSwapchain;
}

impl VulkanApi {
    /// # Safety
    ///
    /// GPU programming is fundamentally unsafe, so all  APIs that interact with the GPU should
    /// be considered unsafe. However,  APIs are only gated by unsafe if they can cause undefined
    /// behavior on the CPU for reasons other than interacting with the GPU.
    pub unsafe fn new(
        window: Option<&dyn HasRawWindowHandle>,
        _api_def: &ApiDef,
        vk_api_def: &ApiDefVulkan,
    ) -> GfxResult<Self> {
        let app_name = vk_api_def.app_name.clone();

        let (require_validation_layers_present, validation_layer_debug_report_flags) =
            match vk_api_def.validation_mode {
                ValidationMode::Disabled => (false, vk::DebugUtilsMessageSeverityFlagsEXT::empty()),
                ValidationMode::EnabledIfAvailable => {
                    (false, vk::DebugUtilsMessageSeverityFlagsEXT::all())
                }
                ValidationMode::Enabled => (true, vk::DebugUtilsMessageSeverityFlagsEXT::all()),
            };

        log::info!("Validation mode: {:?}", vk_api_def.validation_mode);

        let entry = ash::Entry::new()?;
        let instance = VkInstance::new(
            entry,
            window,
            &app_name,
            require_validation_layers_present,
            validation_layer_debug_report_flags,
        )?;

        let inner = Arc::new(VulkanDeviceContextInner::new(&instance)?);
        let device_context = VulkanDeviceContext::new(inner)?;

        Ok(Self {
            instance,
            device_context: Some(device_context),
        })
    }
}
