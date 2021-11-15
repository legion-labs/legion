use ash::vk;

use super::VulkanDeviceContext;
use crate::{Descriptor, DescriptorSetLayoutDef, GfxResult};

#[derive(Clone, Debug)]
pub(crate) struct VulkanDescriptorSetLayout {
    vk_layout: vk::DescriptorSetLayout,
}

impl VulkanDescriptorSetLayout {
    pub fn vk_layout(&self) -> vk::DescriptorSetLayout {
        self.vk_layout
    }

    pub fn new(
        device_context: &VulkanDeviceContext,
        definition: &DescriptorSetLayoutDef,
    ) -> GfxResult<(Self, Vec<Descriptor>, u32)> {
        let mut descriptors = Vec::new();
        let mut vk_bindings = Vec::<vk::DescriptorSetLayoutBinding>::new();
        let mut update_data_count = 0;

        for descriptor_def in &definition.descriptor_defs {
            let vk_descriptor_type = super::internal::shader_resource_type_to_descriptor_type(
                descriptor_def.shader_resource_type,
            );

            let vk_binding = vk::DescriptorSetLayoutBinding::builder()
                .binding(descriptor_def.binding)
                .descriptor_type(vk_descriptor_type)
                .descriptor_count(descriptor_def.array_size_normalized())
                .stage_flags(vk::ShaderStageFlags::ALL)
                .build();

            let descriptor = Descriptor {
                name: descriptor_def.name.clone(),
                binding: descriptor_def.binding,
                shader_resource_type: descriptor_def.shader_resource_type,
                vk_type: vk_descriptor_type,
                element_count: descriptor_def.array_size,
                update_data_offset: update_data_count,
            };
            descriptors.push(descriptor);
            vk_bindings.push(vk_binding);

            update_data_count += vk_binding.descriptor_count;
        }

        let vk_layout = unsafe {
            device_context.device().create_descriptor_set_layout(
                &*vk::DescriptorSetLayoutCreateInfo::builder().bindings(&vk_bindings),
                None,
            )?
        };

        Ok((Self { vk_layout }, descriptors, update_data_count))
    }

    pub fn destroy(&self, device_context: &VulkanDeviceContext) {
        unsafe {
            device_context
                .device()
                .destroy_descriptor_set_layout(self.vk_layout, None);
        }
    }
}
