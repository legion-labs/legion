use ash::vk;

use crate::{Descriptor, DescriptorSetLayout, DeviceContext, GfxResult};

#[derive(Clone, Debug)]
pub(crate) struct VulkanDescriptorSetLayout {
    vk_layout: vk::DescriptorSetLayout,
    vk_image_info_count: u32,
    vk_buffer_info_count: u32,
    typed_flat_indices: Vec<u32>,
}

impl VulkanDescriptorSetLayout {
    pub(crate) fn new(
        device_context: &DeviceContext,
        descriptors: &[Descriptor],
    ) -> GfxResult<Self> {
        let mut vk_bindings = Vec::<vk::DescriptorSetLayoutBinding>::new();
        // let mut flat_index = 0;
        let mut typed_flat_indices = Vec::new();
        let mut vk_image_info_count = 0;
        let mut vk_buffer_info_count = 0;

        for (binding, descriptor) in descriptors.iter().enumerate() {
            let element_count = descriptor.element_count;

            let vk_descriptor_type = super::internal::shader_resource_type_to_descriptor_type(
                descriptor.shader_resource_type,
            );

            let vk_binding = vk::DescriptorSetLayoutBinding::builder()
                .binding(binding as u32)
                .descriptor_type(vk_descriptor_type)
                .descriptor_count(element_count)
                .stage_flags(vk::ShaderStageFlags::ALL)
                .build();

            // let descriptor = Descriptor {
            //     name: descriptor_def.name.clone(),
            //     binding: descriptor_def.binding,
            //     shader_resource_type: descriptor_def.shader_resource_type,
            //     element_count,
            //     flat_index,
            //     typed_flat_index: match descriptor_def.shader_resource_type {
            //         crate::ShaderResourceType::ConstantBuffer
            //         | crate::ShaderResourceType::StructuredBuffer
            //         | crate::ShaderResourceType::RWStructuredBuffer
            //         | crate::ShaderResourceType::ByteAddressBuffer
            //         | crate::ShaderResourceType::RWByteAddressBuffer => {
            //             let offset = vk_buffer_info_count;
            //             vk_buffer_info_count += vk_binding.descriptor_count;
            //             offset
            //         }

            //         crate::ShaderResourceType::Sampler
            //         | crate::ShaderResourceType::Texture2D
            //         | crate::ShaderResourceType::RWTexture2D
            //         | crate::ShaderResourceType::Texture2DArray
            //         | crate::ShaderResourceType::RWTexture2DArray
            //         | crate::ShaderResourceType::Texture3D
            //         | crate::ShaderResourceType::RWTexture3D
            //         | crate::ShaderResourceType::TextureCube
            //         | crate::ShaderResourceType::TextureCubeArray => {
            //             let offset = vk_image_info_count;
            //             vk_image_info_count += vk_binding.descriptor_count;
            //             offset
            //         }
            //     },
            // };

            let typed_flat_index = match descriptor.shader_resource_type {
                crate::ShaderResourceType::ConstantBuffer
                | crate::ShaderResourceType::StructuredBuffer
                | crate::ShaderResourceType::RWStructuredBuffer
                | crate::ShaderResourceType::ByteAddressBuffer
                | crate::ShaderResourceType::RWByteAddressBuffer => {
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

            typed_flat_indices.push(typed_flat_index);

            // flat_index += element_count;
            vk_bindings.push(vk_binding);
        }

        let vk_layout = unsafe {
            device_context.vk_device().create_descriptor_set_layout(
                &*vk::DescriptorSetLayoutCreateInfo::builder().bindings(&vk_bindings),
                None,
            )?
        };

        Ok(Self {
            vk_layout,
            vk_image_info_count,
            vk_buffer_info_count,
            typed_flat_indices,
        })
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

    pub(crate) fn vk_typed_flat_index(&self, descriptor_index: u32) -> u32 {
        self.inner.backend_layout.typed_flat_indices[descriptor_index as usize]
    }
}
