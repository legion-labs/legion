#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanDescriptorHeap;
use crate::{
    deferred_drop::Drc, DescriptorHeapDef, DescriptorSetBufWriter, DescriptorSetLayoutDrc,
    DeviceContextDrc, GfxResult,
};

struct DescriptorHeap {
    device_context: DeviceContextDrc,

    #[cfg(feature = "vulkan")]
    platform_descriptor_heap: VulkanDescriptorHeap,
}

impl Drop for DescriptorHeap {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_descriptor_heap.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct DescriptorHeapDrc {
    inner: Drc<DescriptorHeap>,
}

impl DescriptorHeapDrc {
    pub(crate) fn new(
        device_context: &DeviceContextDrc,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_descriptor_heap = VulkanDescriptorHeap::new(device_context, definition)
            .map_err(|e| {
                log::error!("Error creating descriptor heap {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(DescriptorHeap {
                device_context: device_context.clone(),
                #[cfg(any(feature = "vulkan"))]
                platform_descriptor_heap,
            }),
        })
    }

    pub fn reset(&self) -> GfxResult<()> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.inner
            .platform_descriptor_heap
            .reset(&self.inner.device_context)
    }

    pub fn allocate_descriptor_set(
        &self,
        descriptor_set_layout: &DescriptorSetLayoutDrc,
    ) -> GfxResult<DescriptorSetBufWriter> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.inner
            .platform_descriptor_heap
            .allocate_descriptor_set(&self.inner.device_context, descriptor_set_layout)
    }
}
