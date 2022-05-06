use lgn_graphics_api::{
    DescriptorHeap, DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSetHandle,
    DescriptorSetLayout,
};

use lgn_core::Handle;

pub struct DescriptorPool {
    descriptor_heap_partition: DescriptorHeapPartition,
}

impl DescriptorPool {
    pub(crate) fn new(
        descriptor_heap: &DescriptorHeap,
        heap_partition_def: DescriptorHeapDef,
    ) -> Self {
        Self {
            descriptor_heap_partition: DescriptorHeapPartition::new(
                descriptor_heap,
                true,
                heap_partition_def,
            ),
        }
    }

    pub fn begin_frame(&mut self) {
        self.descriptor_heap_partition.reset().unwrap();
    }

    pub fn descriptor_heap_partition_mut(&self) -> &DescriptorHeapPartition {
        &self.descriptor_heap_partition
    }

    pub fn write_descriptor_set(
        &self,
        layout: &DescriptorSetLayout,
        descriptors: &[DescriptorRef],
    ) -> DescriptorSetHandle {
        self.descriptor_heap_partition
            .write(layout, descriptors)
            .unwrap()
    }
}

pub type DescriptorPoolHandle = Handle<DescriptorPool>;
