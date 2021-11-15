use ash::vk;

use super::VulkanDeviceContext;
use crate::{DeviceContextDrc, GfxResult, ShaderModuleDef};

#[derive(Debug)]
pub(crate) struct VulkanShaderModule {
    shader_module: vk::ShaderModule,
}

impl VulkanShaderModule {
    pub fn new(device_context: &DeviceContextDrc, data: ShaderModuleDef<'_>) -> GfxResult<Self> {
        match data {
            ShaderModuleDef::SpirVBytes(bytes) => {
                Self::new_from_bytes(device_context.platform_device_context(), bytes)
            }
            ShaderModuleDef::Null(_) => unreachable!(),
            //ShaderModuleDef::VkSpvPrepared(spv) => {
            //    VulkanShaderModule::new_from_spv(device_context, spv)
            //}
        }
    }

    pub fn destroy(&self, device_context: &DeviceContextDrc) {
        unsafe {
            device_context
                .platform_device()
                .destroy_shader_module(self.shader_module, None);
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

        Ok(Self { shader_module })
    }

    pub fn vk_shader_module(&self) -> vk::ShaderModule {
        self.shader_module
    }
}
