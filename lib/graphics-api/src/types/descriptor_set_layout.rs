#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanDescriptorSetLayout;

use crate::{
    deferred_drop::Drc, Descriptor, DescriptorSetLayoutDef, DeviceContext, GfxResult,
    MAX_DESCRIPTOR_BINDINGS,
};

pub(crate) struct DescriptorSetLayoutInner {
    device_context: DeviceContext,
    definition: DescriptorSetLayoutDef,
    set_index: u32,
    binding_mask: u64,

    #[cfg(any(feature = "vulkan"))]
    descriptors: Vec<Descriptor>,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_layout: VulkanDescriptorSetLayout,
}

impl Drop for DescriptorSetLayoutInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_layout.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct DescriptorSetLayout {
    pub(crate) inner: Drc<DescriptorSetLayoutInner>,
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

    pub fn binding_mask(&self) -> u64 {
        self.inner.binding_mask
    }

    pub fn find_descriptor_index_by_name(&self, name: &str) -> Option<usize> {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        self.inner
            .descriptors
            .iter()
            .position(|descriptor| name == descriptor.name)
    }

    pub fn descriptor(&self, index: usize) -> &Descriptor {
        #[cfg(not(any(feature = "vulkan")))]
        unimplemented!();

        #[cfg(any(feature = "vulkan"))]
        &self.inner.descriptors[index]
    }

    pub fn new(
        device_context: &DeviceContext,
        definition: &DescriptorSetLayoutDef,
    ) -> GfxResult<Self> {
        let mut binding_mask = 0;
        for descriptor_def in &definition.descriptor_defs {
            assert!((descriptor_def.binding as usize) < MAX_DESCRIPTOR_BINDINGS);
            let mask = 1u64 << descriptor_def.binding;
            assert!((binding_mask & mask) == 0, "Binding already in use");
            binding_mask |= mask;
        }

        #[cfg(feature = "vulkan")]
        let (platform_layout, descriptors) =
            VulkanDescriptorSetLayout::new(device_context, definition).map_err(|e| {
                lgn_telemetry::error!("Error creating platform descriptor set layout {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;

        let result = Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(DescriptorSetLayoutInner {
                    device_context: device_context.clone(),
                    definition: definition.clone(),
                    set_index: definition.frequency,
                    binding_mask,
                    #[cfg(any(feature = "vulkan"))]
                    descriptors,
                    #[cfg(any(feature = "vulkan"))]
                    platform_layout,
                }),
        };

        Ok(result)
    }
}
