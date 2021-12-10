#[cfg(feature = "vulkan")]
use crate::backends::vulkan::{VulkanDescriptorHeap, VulkanDescriptorHeapPartition};

use crate::{
    deferred_drop::Drc, DescriptorHeapDef, DescriptorSetBufWriter, DescriptorSetLayout,
    DeviceContext, GfxResult,
};

//
// DescriptorHeapInner
//

pub(crate) struct DescriptorHeapInner {
    pub(crate) device_context: DeviceContext,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_descriptor_heap: VulkanDescriptorHeap,
}

impl Drop for DescriptorHeapInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_descriptor_heap.destroy(&self.device_context);
    }
}

//
// DescriptorHeap
//

#[derive(Clone)]
pub struct DescriptorHeap {
    pub(crate) inner: Drc<DescriptorHeapInner>,
}

impl DescriptorHeap {
    pub(crate) fn new(
        device_context: &DeviceContext,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_descriptor_heap = VulkanDescriptorHeap::new(device_context, definition)
            .map_err(|e| {
                log::error!("Error creating descriptor heap {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;

        Ok(Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(DescriptorHeapInner {
                    device_context: device_context.clone(),
                    #[cfg(any(feature = "vulkan"))]
                    platform_descriptor_heap,
                }),
        })
    }

    // pub fn reset(&self) -> GfxResult<()> {
    //     self.inner
    //         .partitions
    //         .iter()
    //         .for_each(|x| x.reset().unwrap());
    //     Ok(())
    // }

    pub fn alloc_partition(
        &self,
        transient: bool,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<DescriptorHeapPartition> {
        // #[cfg(any(feature = "vulkan"))]
        // self.alloc_partition_platform(transient, definition)
        DescriptorHeapPartition::new(self.clone(), transient, definition)
    }

    pub fn free_partition(&self, _partition: DescriptorHeapPartition) {
        // #[cfg(any(feature = "vulkan"))]
        // self.free_partition_platform(partition)
    }
}

//
// DescriptorHeapPartitionInner
//

pub(crate) struct DescriptorHeapPartitionInner {
    pub(crate) heap: DescriptorHeap,
    pub(crate) transient: bool,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_descriptor_heap_partition: VulkanDescriptorHeapPartition,
}

impl Drop for DescriptorHeapPartitionInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_descriptor_heap_partition
            .destroy(&self.heap.inner.device_context);
    }
}

//
// DescriptorHeapPartition
//

pub struct DescriptorHeapPartition {
    pub(crate) inner: Box<DescriptorHeapPartitionInner>,
}

impl DescriptorHeapPartition {
    pub(crate) fn new(
        heap: DescriptorHeap,
        transient: bool,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_descriptor_heap_partition =
            VulkanDescriptorHeapPartition::new(&heap.inner.device_context, transient, definition)
                .map_err(|e| {
                log::error!("Error creating descriptor heap {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;

        Ok(Self {
            inner: Box::new(DescriptorHeapPartitionInner {
                heap,
                transient,
                #[cfg(any(feature = "vulkan"))]
                platform_descriptor_heap_partition,
            }),
        })
    }

    pub fn allocate_descriptor_set(
        &self,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> GfxResult<DescriptorSetBufWriter> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.allocate_descriptor_set_platform(descriptor_set_layout)
    }

    pub fn reset(&self) -> GfxResult<()> {
        assert_eq!(self.inner.transient, true);

        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.reset_platform().unwrap();

        Ok(())
    }
}
