use ash::vk;

use crate::backends::deferred_drop::Drc;
use crate::{DescriptorSetLayout, DescriptorSetLayoutDef, GfxError, GfxResult, ShaderResourceType};

use super::{VulkanApi, VulkanDeviceContext};

#[derive(Clone, Debug)]
pub(super) struct VulkanDescriptor {
    pub(super) name: String,
    pub(super) binding: u32,
    pub(super) shader_resource_type: ShaderResourceType,
    pub(super) vk_type: vk::DescriptorType,
    pub(super) element_count: u32,
    pub(super) update_data_offset_in_set: u32,
}

#[derive(Clone, Debug)]
pub struct VulkanDescriptorSetLayoutInner {
    device_context: VulkanDeviceContext,
    set_index: u32,
    update_data_count_per_set: u32,
    descriptors: Vec<VulkanDescriptor>,
    vk_layout: vk::DescriptorSetLayout,
}

impl Drop for VulkanDescriptorSetLayoutInner {
    fn drop(&mut self) {
        unsafe {
            self.device_context
                .device()
                .destroy_descriptor_set_layout(self.vk_layout, None);
        }
    }
}

#[derive(Clone, Debug)]
pub struct VulkanDescriptorSetLayout {
    inner: Drc<VulkanDescriptorSetLayoutInner>,
}

impl VulkanDescriptorSetLayout {
    pub(super) fn device_context(&self) -> &VulkanDeviceContext {
        &self.inner.device_context
    }

    pub(super) fn set_index(&self) -> u32 {
        self.inner.set_index
    }

    pub(super) fn update_data_count_per_set(&self) -> u32 {
        self.inner.update_data_count_per_set
    }

    pub(super) fn vk_layout(&self) -> vk::DescriptorSetLayout {
        self.inner.vk_layout
    }

    pub(super) fn find_descriptor_index_by_name(&self, name: &str) -> Option<u32> {
        self.inner
            .descriptors
            .iter()
            .position(|descriptor| name == descriptor.name)
            .map(|opt| opt as u32)
    }

    pub(super) fn descriptor(&self, index: u32) -> GfxResult<&VulkanDescriptor> {
        self.inner
            .descriptors
            .get(index as usize)
            .ok_or_else(|| GfxError::from("Invalid descriptor index"))
    }

    pub(super) fn new(
        device_context: &VulkanDeviceContext,
        descriptor_set_layout_def: &DescriptorSetLayoutDef,
    ) -> GfxResult<Self> {
        let mut descriptors = Vec::new();
        let mut vk_bindings = Vec::<vk::DescriptorSetLayoutBinding>::new();
        let mut update_data_count_per_set = 0;

        for descriptor_def in &descriptor_set_layout_def.descriptor_defs {
            let vk_descriptor_type = super::internal::shader_resource_type_to_descriptor_type(
                descriptor_def.shader_resource_type,
            );

            let vk_binding = vk::DescriptorSetLayoutBinding::builder()
                .binding(descriptor_def.binding)
                .descriptor_type(vk_descriptor_type)
                .descriptor_count(descriptor_def.array_size_normalized())
                .stage_flags(vk::ShaderStageFlags::ALL)
                .build();

            let descriptor = VulkanDescriptor {
                name: descriptor_def.name.clone(),
                binding: descriptor_def.binding,
                shader_resource_type: descriptor_def.shader_resource_type,
                vk_type: vk_descriptor_type,
                element_count: descriptor_def.array_size,
                update_data_offset_in_set: update_data_count_per_set,
            };
            descriptors.push(descriptor);
            vk_bindings.push(vk_binding);

            update_data_count_per_set += vk_binding.descriptor_count;
        }

        let vk_layout = unsafe {
            device_context.device().create_descriptor_set_layout(
                &*vk::DescriptorSetLayoutCreateInfo::builder().bindings(&vk_bindings),
                None,
            )?
        };

        let result = Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(VulkanDescriptorSetLayoutInner {
                    device_context: device_context.clone(),
                    set_index: descriptor_set_layout_def.frequency,
                    update_data_count_per_set,
                    descriptors,
                    vk_layout,
                }),
        };

        Ok(result)
    }
}

impl DescriptorSetLayout<VulkanApi> for VulkanDescriptorSetLayout {}
