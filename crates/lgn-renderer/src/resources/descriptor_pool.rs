use lgn_graphics_api::{
    DescriptorHeap, DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSetHandle,
    DescriptorSetLayout,
};

use lgn_core::Handle;

use super::OnFrameEventHandler;

pub struct DescriptorPool {
    descriptor_heap_partition: DescriptorHeapPartition,
}

impl DescriptorPool {
    pub(crate) fn new(
        descriptor_heap: &DescriptorHeap,
        heap_partition_def: &DescriptorHeapDef,
    ) -> Self {
        Self {
            descriptor_heap_partition: DescriptorHeapPartition::new(
                descriptor_heap,
                true,
                heap_partition_def,
            )
            .unwrap(),
        }
    }

    pub fn descriptor_heap_partition_mut(&self) -> &DescriptorHeapPartition {
        &self.descriptor_heap_partition
    }

    pub fn write_descriptor_set(
        &self,
        layout: &DescriptorSetLayout,
        descriptors: &[DescriptorRef<'_>],
    ) -> DescriptorSetHandle {
        self.descriptor_heap_partition
            .write(layout, descriptors)
            .unwrap()
    }

    fn reset(&self) {
        self.descriptor_heap_partition.reset().unwrap();
    }
}

impl OnFrameEventHandler for DescriptorPool {
    fn on_begin_frame(&mut self) {
        self.reset();
    }

    fn on_end_frame(&mut self) {}
}

pub type DescriptorPoolHandle = Handle<DescriptorPool>;
