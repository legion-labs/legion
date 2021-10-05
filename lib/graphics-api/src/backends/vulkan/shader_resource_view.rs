use crate::{GfxResult, ShaderResourceView, ShaderResourceViewDef};
use super::{VulkanApi, VulkanBuffer};


#[derive(Clone, Debug)]
pub struct VulkanShaderResourceView {
    
}

impl VulkanShaderResourceView {
    pub fn from_buffer(_buffer: &VulkanBuffer,_def: &ShaderResourceViewDef) -> GfxResult<Self> {
        Ok( VulkanShaderResourceView {            
        }
    )
    }
}

impl ShaderResourceView<VulkanApi> for VulkanShaderResourceView {
    
}