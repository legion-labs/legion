use ash::vk;

use crate::{Descriptor, DescriptorSetLayout, DescriptorSetLayoutDef, DeviceContext, GfxResult};

#[derive(Clone, Debug)]
pub(crate) struct VulkanDescriptorSetLayout {
    vk_layout: vk::DescriptorSetLayout,
    vk_image_info_count: u32,
    vk_buffer_info_count: u32,
}

impl VulkanDescriptorSetLayout {
    pub(crate) fn new(
        device_context: &DeviceContext,
        definition: &DescriptorSetLayoutDef,
    ) -> GfxResult<(Self, Vec<Descriptor>)> {
        let mut descriptors = Vec::new();
        let mut vk_bindings = Vec::<vk::DescriptorSetLayoutBinding>::new();
        // let mut update_data_count = 0;
        let mut vk_image_info_count = 0;
        let mut vk_buffer_info_count = 0;

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

            let mut descriptor = Descriptor {
                name: descriptor_def.name.clone(),
                binding: descriptor_def.binding,
                shader_resource_type: descriptor_def.shader_resource_type,
                // vk_type: vk_descriptor_type,
                element_count: descriptor_def.array_size,
                update_data_offset: 0,
            };

            descriptor.update_data_offset = match descriptor_def.shader_resource_type {
                crate::ShaderResourceType::ConstantBuffer
                | crate::ShaderResourceType::StructuredBuffer
                | crate::ShaderResourceType::RWStructuredBuffer
                | crate::ShaderResourceType::ByteAdressBuffer
                | crate::ShaderResourceType::RWByteAdressBuffer => {
                    let offset = vk_buffer_info_count;
                    vk_buffer_info_count += vk_binding.descriptor_count;
                    offset
                }

                crate::ShaderResourceType::Sampler
                | crate::ShaderResourceType::Texture2D
                | crate::ShaderResourceType::RWTexture2D
                | crate::ShaderResourceType::Texture2DArray
                | crate::ShaderResourceType::RWTexture2DArray
                | crate::ShaderResourceType::Texture3D
                | crate::ShaderResourceType::RWTexture3D
                | crate::ShaderResourceType::TextureCube
                | crate::ShaderResourceType::TextureCubeArray => {
                    let offset = vk_image_info_count;
                    vk_image_info_count += vk_binding.descriptor_count;
                    offset
                }
            };

            descriptors.push(descriptor);
            vk_bindings.push(vk_binding);
        }

        let vk_layout = unsafe {
            device_context.vk_device().create_descriptor_set_layout(
                &*vk::DescriptorSetLayoutCreateInfo::builder().bindings(&vk_bindings),
                None,
            )?
        };

        Ok((
            Self {
                vk_layout,
                vk_image_info_count,
                vk_buffer_info_count,
            },
            descriptors,
        ))
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .destroy_descriptor_set_layout(self.vk_layout, None);
        }
    }
}

impl DescriptorSetLayout {
    pub(crate) fn vk_layout(&self) -> vk::DescriptorSetLayout {
        self.inner.backend_layout.vk_layout
    }

    pub(crate) fn vk_image_info_count(&self) -> u32 {
        self.inner.backend_layout.vk_image_info_count
    }

    pub(crate) fn vk_buffer_info_count(&self) -> u32 {
        self.inner.backend_layout.vk_buffer_info_count
    }
}
