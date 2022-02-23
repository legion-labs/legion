use ash::vk;
use smallvec::SmallVec;

use crate::{
    DescriptorRef, DescriptorSetLayout, DescriptorSetWriter, DeviceContext, GfxResult,
    ShaderResourceType, MAX_DESCRIPTOR_BINDINGS,
};

pub struct VulkanDescriptorSetWriter {
    // vk_image_infos: &'frame mut [vk::DescriptorImageInfo],
// vk_buffer_infos: &'frame mut [vk::DescriptorBufferInfo],
// vk_pending_writes: &'frame mut [vk::WriteDescriptorSet],
}

impl VulkanDescriptorSetWriter {
    pub fn new(descriptor_set_layout: &DescriptorSetLayout) -> GfxResult<Self> {
        if descriptor_set_layout.vk_layout() == vk::DescriptorSetLayout::null() {
            return Err("Descriptor set layout does not exist in this root signature".into());
        }

        // let vk_image_info_count = descriptor_set_layout.vk_image_info_count();
        // let vk_buffer_info_count = descriptor_set_layout.vk_buffer_info_count();
        // let vk_pending_writes = bump.alloc_slice_fill_default::<vk::WriteDescriptorSet>(
        //     descriptor_set_layout.descriptor_count() as usize,
        // );

        // let vk_image_infos =
        //     bump.alloc_slice_fill_default::<vk::DescriptorImageInfo>(vk_image_info_count as usize);

        // let vk_buffer_infos = bump
        //     .alloc_slice_fill_default::<vk::DescriptorBufferInfo>(vk_buffer_info_count as usize);

        Ok(Self {
            // device_context,
            // vk_image_infos,
            // vk_buffer_infos,
            // vk_pending_writes,
        })
    }
}

impl<'a> DescriptorSetWriter<'a> {
    #[allow(clippy::todo)]
    pub fn backend_set_descriptors(
        &mut self,
        device_context: &DeviceContext,
        descriptor_refs: &[DescriptorRef<'_>],
    ) {
        const MAX_BUFFER_DESCRIPTORS: usize = 256;
        const MAX_IMAGE_DESCRIPTORS: usize = 256;

        let descriptor_count = self.descriptor_set_layout.descriptor_count();
        let vk_image_info_count = self.descriptor_set_layout.vk_image_info_count();
        // assert!((vk_image_info_count as usize) < MAX_IMAGE_DESCRIPTORS);
        let vk_buffer_info_count = self.descriptor_set_layout.vk_buffer_info_count();
        // assert!((vk_buffer_info_count as usize) < MAX_BUFFER_DESCRIPTORS);

        let mut vk_pending_writes =
            SmallVec::<[vk::WriteDescriptorSet; MAX_DESCRIPTOR_BINDINGS]>::with_capacity(
                descriptor_count as usize,
            );
        unsafe { vk_pending_writes.set_len(descriptor_count as usize) };

        let mut vk_image_infos =
            SmallVec::<[vk::DescriptorImageInfo; MAX_IMAGE_DESCRIPTORS]>::with_capacity(
                vk_image_info_count as usize,
            );
        unsafe { vk_image_infos.set_len(vk_image_info_count as usize) };

        let mut vk_buffer_infos =
            SmallVec::<[vk::DescriptorBufferInfo; MAX_BUFFER_DESCRIPTORS]>::with_capacity(
                vk_buffer_info_count as usize,
            );
        unsafe { vk_buffer_infos.set_len(vk_buffer_info_count as usize) };

        let descriptor_set = &self.descriptor_set;

        for descriptor_index in 0..descriptor_count {
            let descriptor = self.descriptor_set_layout.descriptor(descriptor_index);
            let element_count = descriptor.element_count.get();
            let update_datas = &descriptor_refs[descriptor.flat_index as usize
                ..descriptor.flat_index as usize + element_count as usize];
            let vk_typed_flat_index = self
                .descriptor_set_layout
                .vk_typed_flat_index(descriptor_index);
            let vk_descriptor_type = super::internal::shader_resource_type_to_descriptor_type(
                descriptor.shader_resource_type,
            );
            let write_descriptor_builder = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set.backend_descriptor_set_handle)
                .dst_binding(descriptor_index)
                .descriptor_type(vk_descriptor_type);

            let mut next_index = vk_typed_flat_index;

            match descriptor.shader_resource_type {
                ShaderResourceType::Sampler => {
                    for update_data in update_datas {
                        if let DescriptorRef::Sampler(sampler) = update_data {
                            let image_info = &mut vk_image_infos[next_index as usize];
                            image_info.sampler = sampler.vk_sampler();
                            image_info.image_view = vk::ImageView::null();
                            image_info.image_layout = vk::ImageLayout::UNDEFINED;
                        } else {
                            unreachable!();
                        }
                        next_index += 1;
                    }

                    // Queue a descriptor write
                    vk_pending_writes[descriptor_index as usize] = write_descriptor_builder
                        .image_info(
                            &vk_image_infos[vk_typed_flat_index as usize..next_index as usize],
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
                            let buffer_info = &mut vk_buffer_infos[next_index as usize];
                            buffer_info.buffer = buffer_view.buffer().vk_buffer();
                            buffer_info.offset = buffer_view.offset();
                            buffer_info.range = buffer_view.size();
                        } else {
                            unreachable!();
                        }
                        next_index += 1;
                    }
                    // Queue a descriptor write
                    vk_pending_writes[descriptor_index as usize] = write_descriptor_builder
                        .buffer_info(
                            &vk_buffer_infos[vk_typed_flat_index as usize..next_index as usize],
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
                            let image_info = &mut vk_image_infos[next_index as usize];

                            image_info.sampler = vk::Sampler::null();
                            image_info.image_view = texture_view.vk_image_view();
                            image_info.image_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                        } else {
                            unreachable!();
                        }
                        next_index += 1;
                    }

                    vk_pending_writes[descriptor_index as usize] = write_descriptor_builder
                        .image_info(
                            &vk_image_infos[vk_typed_flat_index as usize..next_index as usize],
                        )
                        .build();
                }
                ShaderResourceType::RWTexture2D
                | ShaderResourceType::RWTexture2DArray
                | ShaderResourceType::RWTexture3D => {
                    for update_data in update_datas {
                        if let DescriptorRef::TextureView(texture_view) = update_data {
                            assert!(texture_view.is_compatible_with_descriptor(descriptor));
                            let image_info = &mut vk_image_infos[next_index as usize];

                            image_info.sampler = vk::Sampler::null();
                            image_info.image_view = texture_view.vk_image_view();
                            image_info.image_layout = vk::ImageLayout::GENERAL;
                        } else {
                            unreachable!();
                        }
                        next_index += 1;
                    }

                    vk_pending_writes[descriptor_index as usize] = write_descriptor_builder
                        .image_info(
                            &vk_image_infos[vk_typed_flat_index as usize..next_index as usize],
                        )
                        .build();
                }
            }
        }

        unsafe {
            device_context
                .vk_device()
                .update_descriptor_sets(&vk_pending_writes, &[]);
        }
    }

    // pub fn backend_flush(&self) {
    //     unsafe {
    //         device_context
    //             .vk_device()
    //             .update_descriptor_sets(self.backend_write.vk_pending_writes, &[]);
    //     }
    // }
}
