use crate::{DescriptorRef, DescriptorSetHandle, DescriptorSetLayout, DeviceContext, GfxResult};

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanDescriptorSetBufWriter;

pub struct DescriptorSetBufWriter {
    descriptor_set: DescriptorSetHandle,
    descriptor_set_layout: DescriptorSetLayout,

    #[cfg(feature = "vulkan")]
    platform_write: VulkanDescriptorSetBufWriter,
}

impl DescriptorSetBufWriter {
    pub fn new(
        descriptor_set: DescriptorSetHandle,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_write = VulkanDescriptorSetBufWriter::new(descriptor_set_layout)?;

        Ok(Self {
            descriptor_set,
            descriptor_set_layout: descriptor_set_layout.clone(),
            #[cfg(any(feature = "vulkan"))]
            platform_write,
        })
    }

    #[allow(clippy::todo)]
    pub fn set_descriptors<'a>(
        &mut self,
        name: &str,
        descriptor_offset: u32,
        update_datas: &[DescriptorRef<'a>],
    ) -> GfxResult<()> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.platform_write.set_descriptors(
            name,
            descriptor_offset,
            update_datas,
            &self.descriptor_set,
            &self.descriptor_set_layout,
        )
    }

    pub fn flush(mut self, vulkan_device_context: &DeviceContext) -> DescriptorSetHandle {
        #[cfg(any(feature = "vulkan"))]
        self.platform_write.flush(vulkan_device_context);

        self.descriptor_set
    }
}
