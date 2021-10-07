use std::sync::Arc;

use crate::{Buffer, BufferSize, BufferView, BufferViewDef, GfxResult, ResourceUsage};

use super::{VulkanApi, VulkanBuffer};

#[derive(Clone, Debug)]
struct VulkanBufferViewInner {
    buffer : VulkanBuffer,
    offset : u64,
    size : u64,
}

#[derive(Clone, Debug)]
pub struct VulkanBufferView {
    inner: Arc<VulkanBufferViewInner>    
}

impl VulkanBufferView {
    pub fn from_buffer(buffer: &VulkanBuffer, cbv_def: &BufferViewDef) -> GfxResult<Self> {        

        assert!( buffer.buffer_def().usage.intersects(ResourceUsage::HAS_CONST_BUFFER_VIEW) );

        let buffer_size = buffer.buffer_def().size;

        let size = match cbv_def.size {
            BufferSize::InBytes(x) => x.get(),
            BufferSize::WholeSize => buffer_size
        };

        if (cbv_def.offset + size) > buffer_size {
            return Err( "Invalid view.".into() );
        }

        Ok( VulkanBufferView {
            inner: Arc::new( VulkanBufferViewInner{
                buffer: buffer.clone(),     
                offset : cbv_def.offset,
                size
            })
        })    
    }
}

impl BufferView<VulkanApi> for VulkanBufferView {
    fn buffer(&self) -> &VulkanBuffer {
        &self.inner.buffer
    }

    fn offset(&self) -> u64 {
        self.inner.offset
    }

    fn size(&self) -> u64 {
        self.inner.size
    }
}