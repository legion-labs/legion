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

    // pub fn allocate_descriptor_set(
    //     &self,
    //     descriptor_set_layout: &DescriptorSetLayout,
    // ) -> GfxResult<DescriptorSetWriter> {
    //     self.descriptor_heap_partition
    //         .get_writer(descriptor_set_layout)
    // }

    pub fn write_descriptor_set(
        &self,
        // device_context: &DeviceContext,
        layout: &DescriptorSetLayout,
        descriptors: &[DescriptorRef<'_>],
        // descriptor_set: &impl DescriptorSetDataProvider,
        // bump: &'frame Bump,
    ) -> DescriptorSetHandle {
        self.descriptor_heap_partition
            .write(layout, descriptors)
            .unwrap()
        // writer.set_descriptors(descriptors);
        // writer.flush(device_context)
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
