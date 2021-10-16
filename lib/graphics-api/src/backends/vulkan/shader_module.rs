use super::{VulkanApi, VulkanDeviceContext};
use crate::{GfxResult, ShaderModule, ShaderModuleDef};
use ash::vk;
use std::sync::Arc;

#[derive(Debug)]
pub struct ShaderModuleVulkanInner {
    device_context: VulkanDeviceContext,
    shader_module: vk::ShaderModule,
}

impl Drop for ShaderModuleVulkanInner {
    fn drop(&mut self) {
        unsafe {
            self.device_context
                .device()
                .destroy_shader_module(self.shader_module, None);
        }
    }
}

#[derive(Clone, Debug)]
pub struct VulkanShaderModule {
    inner: Arc<ShaderModuleVulkanInner>,
}

impl VulkanShaderModule {
    pub fn new(device_context: &VulkanDeviceContext, data: ShaderModuleDef<'_>) -> GfxResult<Self> {
        match data {
            ShaderModuleDef::SpirVBytes(bytes) => Self::new_from_bytes(device_context, bytes),
            ShaderModuleDef::Null(_) => unreachable!(),
            //ShaderModuleDef::VkSpvPrepared(spv) => {
            //    VulkanShaderModule::new_from_spv(device_context, spv)
            //}
        }
    }

    pub fn new_from_bytes(device_context: &VulkanDeviceContext, data: &[u8]) -> GfxResult<Self> {
        let spv = ash::util::read_spv(&mut std::io::Cursor::new(data))?;
        Self::new_from_spv(device_context, &spv)
    }

    pub fn new_from_spv(device_context: &VulkanDeviceContext, data: &[u32]) -> GfxResult<Self> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(data);

        let shader_module = unsafe {
            device_context
                .device()
                .create_shader_module(&create_info, None)?
        };
        let inner = ShaderModuleVulkanInner {
            device_context: device_context.clone(),
            shader_module,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn vk_shader_module(&self) -> vk::ShaderModule {
        self.inner.shader_module
    }
}

impl ShaderModule<VulkanApi> for VulkanShaderModule {}
