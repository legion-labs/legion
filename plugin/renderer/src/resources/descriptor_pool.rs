use graphics_api::{
    DescriptorHeap, DescriptorHeapDef, DescriptorSetBufWriter, DescriptorSetLayout, DeviceContext,
    GfxResult,
};

use super::OnFrameEventHandler;
use crate::RenderHandle;

pub(crate) struct DescriptorPool {
    heap: DescriptorHeap,
}

impl DescriptorPool {
    pub(crate) fn new(device_context: &DeviceContext, heap_def: &DescriptorHeapDef) -> Self {
        Self {
            heap: device_context.create_descriptor_heap(heap_def).unwrap(),
        }
    }

    pub(crate) fn allocate_descriptor_set(
        &mut self,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> GfxResult<DescriptorSetBufWriter> {
        self.heap.allocate_descriptor_set(descriptor_set_layout)
    }

    pub(crate) fn reset(&mut self) {
        self.heap.reset().unwrap();
    }
}

impl OnFrameEventHandler for DescriptorPool {
    fn on_begin_frame(&mut self) {
        self.reset();
    }

    fn on_end_frame(&mut self) {}
}

pub(crate) type DescriptorPoolHandle = RenderHandle<DescriptorPool>;
