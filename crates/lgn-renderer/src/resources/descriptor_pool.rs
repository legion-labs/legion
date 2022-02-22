use bumpalo::Bump;
use lgn_graphics_api::{
    DescriptorHeap, DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSetHandle,
    DescriptorSetLayout, DescriptorSetWriter, DeviceContext, GfxResult,
};

use lgn_core::Handle;

use super::OnFrameEventHandler;

pub struct DescriptorPool {
    descriptor_heap: DescriptorHeap,
    descriptor_heap_partition: Handle<DescriptorHeapPartition>,
}

impl DescriptorPool {
    pub(crate) fn new(
        descriptor_heap: DescriptorHeap,
        heap_partition_def: &DescriptorHeapDef,
    ) -> Self {
        let descriptor_heap_partition = Handle::new(
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
        device_context: &DeviceContext,
        layout: &DescriptorSetLayout,
        descriptors: &[DescriptorRef<'_>],
        // descriptor_set: &impl DescriptorSetDataProvider,
        bump: &'frame Bump,
    ) -> DescriptorSetHandle {
        let mut writer = self
            .descriptor_heap_partition
            .get_writer(layout, bump)
            .unwrap();
        writer.set_descriptors(descriptors);
        writer.flush(device_context)
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

pub type DescriptorPoolHandle = Handle<DescriptorPool>;
