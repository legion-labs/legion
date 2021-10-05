use crate::{GfxResult, UnorderedAccessViewDef, UnorderedAccessView};
use super::{VulkanApi, VulkanBuffer};

#[derive(Clone, Debug)]
pub struct VulkanUnorderedAccessView {
}

impl VulkanUnorderedAccessView {
    pub fn from_buffer(_buffer: &VulkanBuffer, _def: &UnorderedAccessViewDef) -> GfxResult<Self> {
        Ok( VulkanUnorderedAccessView {            
        }
    )
    }
}

impl UnorderedAccessView<VulkanApi> for VulkanUnorderedAccessView {
    
}