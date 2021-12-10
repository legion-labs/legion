use lgn_graphics_api::{
    DescriptorHeap, DescriptorHeapDef, DescriptorHeapPartition, DescriptorSetBufWriter,
    DescriptorSetLayout, DeviceContext, GfxResult,
};

use super::OnFrameEventHandler;
use crate::RenderHandle;

pub(crate) struct DescriptorPool {
    descriptor_heap: DescriptorHeap,
    descriptor_heap_partition: RenderHandle<DescriptorHeapPartition>,
}

impl DescriptorPool {
    pub(crate) fn new(
        descriptor_heap: DescriptorHeap,
        heap_partition_def: &DescriptorHeapDef,
    ) -> Self {
        let descriptor_heap_partition = RenderHandle::new(
            descriptor_heap
                .alloc_partition(true, heap_partition_def)
                .unwrap(),
        );
        Self {
            descriptor_heap,
            descriptor_heap_partition,
        }
    }

    pub(crate) fn allocate_descriptor_set(
        &mut self,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> GfxResult<DescriptorSetBufWriter> {
        self.descriptor_heap_partition
            .allocate_descriptor_set(descriptor_set_layout)
    }

    fn reset(&self) {
        self.descriptor_heap_partition.reset().unwrap();
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        self.descriptor_heap
            .free_partition(self.descriptor_heap_partition.take());
    }
}

impl OnFrameEventHandler for DescriptorPool {
    fn on_begin_frame(&mut self) {
        self.reset();
    }

    fn on_end_frame(&mut self) {}
}

pub(crate) type DescriptorPoolHandle = RenderHandle<DescriptorPool>;
