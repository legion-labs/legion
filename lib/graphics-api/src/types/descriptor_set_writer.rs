use log::error;

use crate::{
    DescriptorRef, DescriptorSetHandle, DescriptorSetLayout, DeviceContext, GfxError, GfxResult,
    MAX_DESCRIPTOR_BINDINGS,
};

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanDescriptorSetWriter;

pub struct DescriptorSetWriter<'frame> {
    pub(crate) descriptor_set: DescriptorSetHandle,
    pub(crate) descriptor_set_layout: DescriptorSetLayout,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_write: VulkanDescriptorSetWriter<'frame>,

    write_mask: u64, // max number of bindings: 64
}

impl<'frame> DescriptorSetWriter<'frame> {
    pub fn new(
        descriptor_set: DescriptorSetHandle,
        descriptor_set_layout: &DescriptorSetLayout,
        bump: &'frame bumpalo::Bump,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_write = VulkanDescriptorSetWriter::new(descriptor_set_layout, bump)?;

        Ok(Self {
            descriptor_set,
            descriptor_set_layout: descriptor_set_layout.clone(),
            #[cfg(any(feature = "vulkan"))]
            platform_write,
            write_mask: descriptor_set_layout.binding_mask(),
        })
    }

    #[allow(clippy::todo)]
    pub fn set_descriptors_by_name(
        &mut self,
        name: &str,
        update_datas: &[DescriptorRef<'frame>],
    ) -> GfxResult<()> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        let descriptor_index = self
            .descriptor_set_layout
            .find_descriptor_index_by_name(name)
            .ok_or_else(|| GfxError::from("Invalid descriptor name"))?;

        #[cfg(any(feature = "vulkan"))]
        self.set_descriptors_by_index(descriptor_index, update_datas)
    }

    #[allow(clippy::todo)]
    pub fn set_descriptors_by_index(
        &mut self,
        index: usize,
        update_datas: &[DescriptorRef<'frame>],
    ) -> GfxResult<()> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        self.write_mask &= !(1u64 << index);

        #[cfg(any(feature = "vulkan"))]
        self.set_descriptors_by_index_platform(index, update_datas)
    }

    pub fn flush(self, vulkan_device_context: &DeviceContext) -> DescriptorSetHandle {
        if self.write_mask != 0 {
            error!(
                "An instance of DescriptorSetWriter cannot be flushed due to missing descriptors"
            );
            for i in 0..MAX_DESCRIPTOR_BINDINGS {
                let mask = 1u64 << i;
                if (self.write_mask & mask) != 0 {
                    error!("{:?}", self.descriptor_set_layout.descriptor(i));
                }
            }
            panic!();
        }

        #[cfg(any(feature = "vulkan"))]
        self.flush_platform(vulkan_device_context);
        self.descriptor_set
    }
}
