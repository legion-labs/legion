use bumpalo::Bump;
use lgn_graphics_api::{
    DescriptorHeap, DescriptorHeapDef, DescriptorHeapPartition, DescriptorSetDataProvider,
    DescriptorSetHandle, DescriptorSetLayout, DescriptorSetWriter, GfxResult,
};

use super::OnFrameEventHandler;
use crate::RenderHandle;

pub struct DescriptorPool {
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

    pub fn descriptor_heap_partition_mut(&self) -> &DescriptorHeapPartition {
        &self.descriptor_heap_partition
    }

    pub fn allocate_descriptor_set<'frame>(
        &self,
        descriptor_set_layout: &DescriptorSetLayout,
        bump: &'frame Bump,
    ) -> GfxResult<DescriptorSetWriter<'frame>> {
        self.descriptor_heap_partition
            .get_writer(descriptor_set_layout, bump)
    }

    pub fn write_descriptor_set<'frame>(
        &self,
        descriptor_set: &impl DescriptorSetDataProvider,
        bump: &'frame Bump,
    ) -> GfxResult<DescriptorSetHandle> {
        self.descriptor_heap_partition.write(descriptor_set, bump)
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

pub type DescriptorPoolHandle = RenderHandle<DescriptorPool>;
