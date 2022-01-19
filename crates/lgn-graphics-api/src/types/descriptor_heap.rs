#![allow(unsafe_code)]

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::{VulkanDescriptorHeap, VulkanDescriptorHeapPartition};
use crate::{
    deferred_drop::Drc, DescriptorHeapDef, DescriptorRef, DescriptorSetHandle, DescriptorSetLayout,
    DescriptorSetWriter, DeviceContext, GfxResult,
};

//
// DescriptorSetDataProvider
//

pub trait DescriptorSetDataProvider {
    fn layout(&self) -> &'static DescriptorSetLayout;
    fn frequency(&self) -> u32;
    fn descriptor_refs(&self, descriptor_index: usize) -> &[DescriptorRef<'_>];
}

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
                lgn_tracing::error!("Error creating descriptor heap {:?}", e);
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

    pub fn alloc_partition(
        &self,
        transient: bool,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<DescriptorHeapPartition> {
        // todo(vdbdd): is there enough room inside this heap to allocate this partition
        DescriptorHeapPartition::new(self.clone(), transient, definition)
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::todo)]
    pub fn free_partition(&self, _partition: DescriptorHeapPartition) {
        // todo(vdbdd): free
        
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
                lgn_tracing::error!("Error creating descriptor heap {:?}", e);
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

    pub fn reset(&self) -> GfxResult<()> {
        assert!(self.inner.transient);

        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.reset_platform()
    }

    pub fn get_writer<'frame>(
        &self,
        descriptor_set_layout: &DescriptorSetLayout,
        bump: &'frame bumpalo::Bump,
    ) -> GfxResult<DescriptorSetWriter<'frame>> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.get_writer_platform(descriptor_set_layout, bump)
    }

    pub fn write<'frame>(
        &self,
        descriptor_set: &impl DescriptorSetDataProvider,
        bump: &'frame bumpalo::Bump,
    ) -> GfxResult<DescriptorSetHandle> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.write_platform(descriptor_set, bump)
    }
}
