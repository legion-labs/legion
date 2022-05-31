use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ash::vk;

use crate::{DeviceContext, GfxResult, ShaderModule, ShaderModuleDef};

#[derive(Debug)]
pub(crate) struct VulkanShaderModule {
    shader_module: vk::ShaderModule,
    spv_hash: u64,
}

impl PartialEq for VulkanShaderModule {
    fn eq(&self, other: &Self) -> bool {
        self.spv_hash == other.spv_hash
    }
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

    fn new_from_bytes(device_context: &DeviceContext, data: &[u8]) -> GfxResult<Self> {
        let spv = ash::util::read_spv(&mut std::io::Cursor::new(data))?;
        Self::new_from_spv(device_context, &spv)
    }

    fn new_from_spv(device_context: &DeviceContext, data: &[u32]) -> GfxResult<Self> {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let spv_hash = hasher.finish();

        let create_info = vk::ShaderModuleCreateInfo::builder().code(data);

        let shader_module = unsafe {
            device_context
                .vk_device()
                .create_shader_module(&create_info, None)?
        };

        Ok(Self {
            shader_module,
            spv_hash,
        })
    }
}

impl ShaderModule {
    pub(crate) fn vk_shader_module(&self) -> vk::ShaderModule {
        self.inner.backend_shader_module.shader_module
    }
}
