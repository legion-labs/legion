use std::sync::Arc;

use crate::{Buffer, BufferSize, ConstantBufferView, ConstantBufferViewDef, GfxResult, ResourceUsage};

use super::{VulkanApi, VulkanBuffer};


#[derive(Clone, Debug)]
struct VulkanConstantBufferViewInner {
    pub(super) buffer : VulkanBuffer,
    pub(super) offset : u64,
    pub(super) size : u64,
}

#[derive(Clone, Debug)]
pub struct VulkanConstantBufferView {
    inner: Arc<VulkanConstantBufferViewInner>
    // pub(super) vk_view: vk::BufferView,
    
}

impl VulkanConstantBufferView {
    pub fn from_buffer(buffer: &VulkanBuffer, cbv_def: &ConstantBufferViewDef) -> GfxResult<Self> {        

        assert!( buffer.buffer_def().usage.intersects(ResourceUsage::HAS_CONST_BUFFER_VIEW) );

        // let create_info = vk::BufferViewCreateInfo::builder()
        //     .buffer(buffer.vk_buffer())
        //     // .format(buffer.buffer_def.format.into())
        //     .offset(
        //         cbv_def.offset,
        //     )
        //     .range(
        //         cbv_def.size,
        //     );
        
        // let vk_view = 
        // unsafe {
        //     buffer.
        //     device_context().
        //     device().
        //     create_buffer_view(&*create_info, None)?                
        // };

        let buffer_size = buffer.buffer_def().size;

        let size = match cbv_def.size {
            BufferSize::InBytes(x) => x.get(),
            BufferSize::WholeSize => buffer_size
        };

        if (cbv_def.offset + size) > buffer_size {
            return Err( "Invalid view.".into() );
        }

        Ok( VulkanConstantBufferView {
            inner: Arc::new( VulkanConstantBufferViewInner{
                buffer: buffer.clone(),     
                offset : cbv_def.offset,
                size
            })
        })    
    }
}

impl ConstantBufferView<VulkanApi> for VulkanConstantBufferView {
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