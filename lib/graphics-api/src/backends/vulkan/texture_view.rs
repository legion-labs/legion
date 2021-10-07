use std::sync::Arc;

use crate::{TextureView, VulkanApi};

use super::VulkanTexture;


#[derive(Clone, Debug)]
struct VulkanTextureViewInner {
    texture : VulkanTexture,
}

#[derive(Clone, Debug)]
pub struct VulkanTextureView {
    inner: Arc<VulkanTextureViewInner>    
}

impl TextureView<VulkanApi> for VulkanTextureView {
    fn texture(&self) -> &VulkanTexture {
        &self.inner.texture
    }
}