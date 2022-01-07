#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanShaderModule;
use crate::{deferred_drop::Drc, DeviceContext, GfxResult, ShaderModuleDef};

pub(crate) struct ShaderModuleInner {
    device_context: DeviceContext,

    #[cfg(feature = "vulkan")]
    platform_shader_module: VulkanShaderModule,
}

impl Drop for ShaderModuleInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_shader_module.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct ShaderModule {
    inner: Drc<ShaderModuleInner>,
}

impl ShaderModule {
    pub fn new(device_context: &DeviceContext, data: ShaderModuleDef<'_>) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_shader_module =
            VulkanShaderModule::new(device_context, data).map_err(|e| {
                lgn_telemetry::error!("Error creating vulkan shader module {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;

        Ok(Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(ShaderModuleInner {
                    device_context: device_context.clone(),
                    #[cfg(any(feature = "vulkan"))]
                    platform_shader_module,
                }),
        })
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_shader_module(&self) -> &VulkanShaderModule {
        &self.inner.platform_shader_module
    }
}
