use ash::vk;

use crate::{
    DescriptorRef, DescriptorSetLayout, DescriptorSetWriter, DeviceContext, GfxResult,
    ShaderResourceType,
};

pub struct VulkanDescriptorSetWriter<'frame> {
    vk_image_infos: &'frame mut [vk::DescriptorImageInfo],
    vk_buffer_infos: &'frame mut [vk::DescriptorBufferInfo],
    vk_pending_writes: &'frame mut [vk::WriteDescriptorSet],
}

impl<'frame> VulkanDescriptorSetWriter<'frame> {
    pub fn new(
        descriptor_set_layout: &DescriptorSetLayout,
        bump: &'frame bumpalo::Bump,
    ) -> GfxResult<Self> {
        if descriptor_set_layout.vk_layout() == vk::DescriptorSetLayout::null() {
            return Err("Descriptor set layout does not exist in this root signature".into());
        }

        let vk_image_info_count = descriptor_set_layout.vk_image_info_count();
        let vk_buffer_info_count = descriptor_set_layout.vk_buffer_info_count();
        let vk_pending_writes = bump.alloc_slice_fill_default::<vk::WriteDescriptorSet>(
            descriptor_set_layout.descriptor_count() as usize,
        );

        let vk_image_infos =
            bump.alloc_slice_fill_default::<vk::DescriptorImageInfo>(vk_image_info_count as usize);

        let vk_buffer_infos = bump
            .alloc_slice_fill_default::<vk::DescriptorBufferInfo>(vk_buffer_info_count as usize);

        Ok(Self {
            vk_image_infos,
            vk_buffer_infos,
            vk_pending_writes,
        })
    }
}

impl<'frame> DescriptorSetWriter<'frame> {
    #[allow(clippy::todo)]
    pub fn backend_set_descriptors_by_index(
        &mut self,
        descriptor_index: u32,
        update_datas: &[DescriptorRef<'_>],
    ) {
        let descriptor = self.descriptor_set_layout.descriptor(descriptor_index);
        let descriptor_binding = descriptor_index;
        assert!((descriptor.element_count_normalized() as usize) == update_datas.len());

        let descriptor_first_update_data = self
            .descriptor_set_layout
            .vk_typed_flat_index(descriptor_index);
        let descriptor_set = &self.descriptor_set;
        let vk_descriptor_type = super::internal::shader_resource_type_to_descriptor_type(
            descriptor.shader_resource_type,
        );
        let write_descriptor_builder = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set.backend_descriptor_set_handle)
            .dst_binding(descriptor_binding)
            .descriptor_type(vk_descriptor_type);

        let mut next_index = descriptor_first_update_data;

        match descriptor.shader_resource_type {
            ShaderResourceType::Sampler => {
                for update_data in update_datas {
                    if let DescriptorRef::Sampler(sampler) = update_data {
                        let image_info =
                            &mut self.backend_write.vk_image_infos[next_index as usize];
                        image_info.sampler = sampler.vk_sampler();
                        image_info.image_view = vk::ImageView::null();
                        image_info.image_layout = vk::ImageLayout::UNDEFINED;
                    } else {
                        unreachable!();
                    }
                    next_index += 1;
                }

                // Queue a descriptor write
                self.backend_write.vk_pending_writes[descriptor_index as usize] =
                    write_descriptor_builder
                        .image_info(
                            &self.backend_write.vk_image_infos
                                [descriptor_first_update_data as usize..next_index as usize],
                        )
                        .build();
            }

            ShaderResourceType::ConstantBuffer
            | ShaderResourceType::StructuredBuffer
            | ShaderResourceType::RWStructuredBuffer
            | ShaderResourceType::ByteAddressBuffer
            | ShaderResourceType::RWByteAddressBuffer => {
                for update_data in update_datas {
                    if let DescriptorRef::BufferView(buffer_view) = update_data {
                        assert!(buffer_view.is_compatible_with_descriptor(descriptor));
                        let buffer_info =
                            &mut self.backend_write.vk_buffer_infos[next_index as usize];
                        buffer_info.buffer = buffer_view.buffer().vk_buffer();
                        buffer_info.offset = buffer_view.offset();
                        buffer_info.range = buffer_view.size();
                    } else {
                        unreachable!();
                    }
                    next_index += 1;
                }
                // Queue a descriptor write
                self.backend_write.vk_pending_writes[descriptor_index as usize] =
                    write_descriptor_builder
                        .buffer_info(
                            &self.backend_write.vk_buffer_infos
                                [descriptor_first_update_data as usize..next_index as usize],
                        )
                        .build();
            }
            ShaderResourceType::Texture2D
            | ShaderResourceType::Texture2DArray
            | ShaderResourceType::Texture3D
            | ShaderResourceType::TextureCube
            | ShaderResourceType::TextureCubeArray => {
                for update_data in update_datas {
                    if let DescriptorRef::TextureView(texture_view) = update_data {
                        assert!(texture_view.is_compatible_with_descriptor(descriptor));
                        let image_info =
                            &mut self.backend_write.vk_image_infos[next_index as usize];

                        image_info.sampler = vk::Sampler::null();
                        image_info.image_view = texture_view.vk_image_view();
                        image_info.image_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                    } else {
                        unreachable!();
                    }
                    next_index += 1;
                }

                self.backend_write.vk_pending_writes[descriptor_index as usize] =
                    write_descriptor_builder
                        .image_info(
                            &self.backend_write.vk_image_infos
                                [descriptor_first_update_data as usize..next_index as usize],
                        )
                        .build();
            }
            ShaderResourceType::RWTexture2D
            | ShaderResourceType::RWTexture2DArray
            | ShaderResourceType::RWTexture3D => {
                for update_data in update_datas {
                    if let DescriptorRef::TextureView(texture_view) = update_data {
                        assert!(texture_view.is_compatible_with_descriptor(descriptor));
                        let image_info =
                            &mut self.backend_write.vk_image_infos[next_index as usize];

                        image_info.sampler = vk::Sampler::null();
                        image_info.image_view = texture_view.vk_image_view();
                        image_info.image_layout = vk::ImageLayout::GENERAL;
                    } else {
                        unreachable!();
                    }
                    next_index += 1;
                }

                self.backend_write.vk_pending_writes[descriptor_index as usize] =
                    write_descriptor_builder
                        .image_info(
                            &self.backend_write.vk_image_infos
                                [descriptor_first_update_data as usize..next_index as usize],
                        )
                        .build();
            }
        }
    }

    pub fn backend_flush(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .update_descriptor_sets(self.backend_write.vk_pending_writes, &[]);
        }
    }
}
