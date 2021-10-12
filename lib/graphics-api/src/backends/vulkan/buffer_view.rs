use std::sync::Arc;

use crate::{BufferView, BufferViewDef, GfxResult};

use super::{VulkanApi, VulkanBuffer};

#[derive(Clone, Debug)]
struct VulkanBufferViewInner {
    view_def: BufferViewDef,
    buffer: VulkanBuffer,    
}

#[derive(Clone, Debug)]
pub struct VulkanBufferView {
    inner: Arc<VulkanBufferViewInner>    
}

impl VulkanBufferView {
    pub fn from_buffer(buffer: &VulkanBuffer, view_def: &BufferViewDef) -> GfxResult<Self> {        

        view_def.verify::<VulkanApi>(buffer);

        Ok( VulkanBufferView {
            inner: Arc::new( VulkanBufferViewInner{
                buffer: buffer.clone(),     
                view_def: view_def.clone(),                
            })
        })    
    }
}

impl BufferView<VulkanApi> for VulkanBufferView {
    
    fn view_def(&self) -> &BufferViewDef {
        &self.inner.view_def
    }

    fn buffer(&self) -> &VulkanBuffer {
        &self.inner.buffer
    }
}