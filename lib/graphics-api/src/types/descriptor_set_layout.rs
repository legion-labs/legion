#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanDescriptorSetLayout;
#[cfg(any(feature = "vulkan"))]
use crate::GfxError;
use crate::{deferred_drop::Drc, Descriptor, DescriptorSetLayoutDef, DeviceContext, GfxResult};

#[derive(Clone)]
pub(crate) struct DescriptorSetLayoutInner {
    device_context: DeviceContext,
    definition: DescriptorSetLayoutDef,
    set_index: u32,
    update_data_count: u32,

    #[cfg(any(feature = "vulkan"))]
    descriptors: Vec<Descriptor>,

    #[cfg(feature = "vulkan")]
    platform_layout: VulkanDescriptorSetLayout,
}

impl Drop for DescriptorSetLayoutInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_layout.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct DescriptorSetLayout {
    inner: Drc<DescriptorSetLayoutInner>,
}

impl DescriptorSetLayout {
    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn definition(&self) -> &DescriptorSetLayoutDef {
        &self.inner.definition
    }

    pub fn set_index(&self) -> u32 {
        self.inner.set_index
    }

    pub fn update_data_count(&self) -> u32 {
        self.inner.update_data_count
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_layout(&self) -> &VulkanDescriptorSetLayout {
        &self.inner.platform_layout
    }

    pub fn find_descriptor_index_by_name(&self, name: &str) -> Option<u32> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.inner
            .descriptors
            .iter()
            .position(|descriptor| name == descriptor.name)
            .map(|opt| opt as u32)
    }

    pub fn descriptor(&self, index: u32) -> GfxResult<&Descriptor> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.inner
            .descriptors
            .get(index as usize)
            .ok_or_else(|| GfxError::from("Invalid descriptor index"))
    }

    pub fn new(
        device_context: &DeviceContext,
        definition: &DescriptorSetLayoutDef,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let (platform_layout, descriptors, update_data_count) =
            VulkanDescriptorSetLayout::new(device_context, definition).map_err(|e| {
                log::error!("Error creating platform descriptor set layout {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;
        #[cfg(not(any(feature = "vulkan")))]
        let update_data_count = 0;

        let result = Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(DescriptorSetLayoutInner {
                    device_context: device_context.clone(),
                    definition: definition.clone(),
                    set_index: definition.frequency,
                    update_data_count,
                    #[cfg(any(feature = "vulkan"))]
                    descriptors,
                    #[cfg(any(feature = "vulkan"))]
                    platform_layout,
                }),
        };

        Ok(result)
    }
}
