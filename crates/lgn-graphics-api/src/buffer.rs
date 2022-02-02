#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanBuffer;
use crate::{deferred_drop::Drc, BufferDef, BufferView, BufferViewDef, DeviceContext};

pub(crate) struct BufferInner {
    pub(crate) buffer_def: BufferDef,
    pub(crate) device_context: DeviceContext,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_buffer: VulkanBuffer,
}

impl Drop for BufferInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_buffer
            .destroy(&self.device_context, &self.buffer_def);
    }
}

#[derive(Clone)]
pub struct Buffer {
    pub(crate) inner: Drc<BufferInner>,
}

impl Buffer {
    pub fn new(device_context: &DeviceContext, buffer_def: &BufferDef) -> Self {
        #[cfg(feature = "vulkan")]
        let platform_buffer = VulkanBuffer::new(device_context, buffer_def);

        Self {
            inner: device_context.deferred_dropper().new_drc(BufferInner {
                device_context: device_context.clone(),
                buffer_def: *buffer_def,
                #[cfg(any(feature = "vulkan"))]
                platform_buffer,
            }),
        }
    }

    pub fn definition(&self) -> &BufferDef {
        &self.inner.buffer_def
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn required_alignment(&self) -> u64 {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.required_alignment_platform()
    }

    pub fn create_view(&self, view_def: &BufferViewDef) -> BufferView {
        BufferView::from_buffer(self, view_def)
    }
}

#[cfg(feature = "vulkan")]
pub type BufferCopy = ash::vk::BufferCopy;

#[cfg(not(any(feature = "vulkan")))]
pub struct BufferCopy {}
