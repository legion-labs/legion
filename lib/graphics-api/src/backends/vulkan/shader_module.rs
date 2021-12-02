use ash::vk;

use crate::{DeviceContext, GfxResult, ShaderModuleDef};

#[derive(Debug)]
pub(crate) struct VulkanShaderModule {
    shader_module: vk::ShaderModule,
}

impl VulkanShaderModule {
    pub fn new(device_context: &DeviceContext, data: ShaderModuleDef<'_>) -> GfxResult<Self> {
        match data {
            ShaderModuleDef::SpirVBytes(bytes) => Self::new_from_bytes(device_context, bytes),
            ShaderModuleDef::Null(_) => unreachable!(),
        }
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .destroy_shader_module(self.shader_module, None);
        }
    }

    pub fn new_from_bytes(device_context: &DeviceContext, data: &[u8]) -> GfxResult<Self> {
        let spv = ash::util::read_spv(&mut std::io::Cursor::new(data))?;
        Self::new_from_spv(device_context, &spv)
    }

    pub fn new_from_spv(device_context: &DeviceContext, data: &[u32]) -> GfxResult<Self> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(data);

        let shader_module = unsafe {
            device_context
                .vk_device()
                .create_shader_module(&create_info, None)?
        };

        Ok(Self { shader_module })
    }

    pub fn vk_shader_module(&self) -> vk::ShaderModule {
        self.shader_module
    }
}
