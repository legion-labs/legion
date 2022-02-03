use crate::{
    backends::BackendBuffer, deferred_drop::Drc, BufferDef, BufferView, BufferViewDef,
    DeviceContext,
};

pub(crate) struct BufferInner {
    pub(crate) buffer_def: BufferDef,
    pub(crate) device_context: DeviceContext,
    pub(crate) backend_buffer: BackendBuffer,
}

impl Drop for BufferInner {
    fn drop(&mut self) {
        self.backend_buffer
            .destroy(&self.device_context, &self.buffer_def);
    }
}

#[derive(Clone)]
pub struct Buffer {
    pub(crate) inner: Drc<BufferInner>,
}

impl Buffer {
    pub fn new(device_context: &DeviceContext, buffer_def: &BufferDef) -> Self {
        let platform_buffer = BackendBuffer::new(device_context, buffer_def);

        Self {
            inner: device_context.deferred_dropper().new_drc(BufferInner {
                device_context: device_context.clone(),
                buffer_def: *buffer_def,
                backend_buffer: platform_buffer,
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
        self.backend_required_alignment()
    }

    pub fn create_view(&self, view_def: &BufferViewDef) -> BufferView {
        BufferView::from_buffer(self, view_def)
    }
}

pub struct BufferCopy {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64,
}
