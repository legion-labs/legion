use ash::vk;

use crate::{
    BufferView, DescriptorRef, DescriptorSetBufWriter, GfxError, GfxResult, ShaderResourceType,
    VulkanApi,
};

use super::{VulkanDescriptorSetHandle, VulkanDescriptorSetLayout};

struct VkDescriptors {
    // one per set * elements in each descriptor
    image_infos: Vec<vk::DescriptorImageInfo>,
    buffer_infos: Vec<vk::DescriptorBufferInfo>,
}

impl VkDescriptors {
    fn new(update_data_count: u32) -> Self {
        Self {
            image_infos: vec![vk::DescriptorImageInfo::default(); update_data_count as usize],
            buffer_infos: vec![vk::DescriptorBufferInfo::default(); update_data_count as usize],
        }
    }
}

pub struct VulkanDescriptorSetBufWriter {
    descriptor_set: VulkanDescriptorSetHandle,
    descriptor_set_layout: VulkanDescriptorSetLayout,
    vk_descriptors: VkDescriptors,
    pending_writes: Vec<vk::WriteDescriptorSet>,
}

impl VulkanDescriptorSetBufWriter {
    pub fn new(
        descriptor_set: VulkanDescriptorSetHandle,
        descriptor_set_layout: &VulkanDescriptorSetLayout,
    ) -> GfxResult<Self> {
        if descriptor_set_layout.vk_layout() == vk::DescriptorSetLayout::null() {
            return Err("Descriptor set layout does not exist in this root signature".into());
        }

        let update_data_count = descriptor_set_layout.update_data_count();
        let vk_descriptors = VkDescriptors::new(update_data_count);
        let pending_writes = Vec::with_capacity(update_data_count as usize);

        Ok(Self {
            descriptor_set,
            descriptor_set_layout: descriptor_set_layout.clone(),
            vk_descriptors,
            pending_writes,
        })
    }
}

impl DescriptorSetBufWriter<VulkanApi> for VulkanDescriptorSetBufWriter {
    fn set_descriptors<'a>(
        &mut self,
        name: &str,
        descriptor_offset: u32,
        update_datas: &[DescriptorRef<'a, VulkanApi>],
    ) -> GfxResult<()> {
        let layout = &self.descriptor_set_layout;
        let descriptor_index = layout
            .find_descriptor_index_by_name(name)
            .ok_or(GfxError::from("toto"))?;
        let descriptor = layout.descriptor(descriptor_index)?;
        assert!(
            descriptor_offset as usize + update_datas.len() <= descriptor.element_count as usize
        );
        let descriptor_first_update_data = descriptor.update_data_offset;
        let descriptorset = &self.descriptor_set;
        let write_descriptor_builder = vk::WriteDescriptorSet::builder()
            .dst_set(descriptorset.0)
            .dst_binding(descriptor.binding)
            .dst_array_element(descriptor_offset)
            .descriptor_type(descriptor.vk_type);

        let begin_index = descriptor_first_update_data + descriptor_offset;
        let mut next_index = begin_index;

        match descriptor.shader_resource_type {
            ShaderResourceType::Sampler => {
                // assert!(matches!(update_datas, DescriptorUpdateData::Sampler { .. }));
                for update_data in update_datas {
                    if let DescriptorRef::Sampler(sampler) = update_data {
                        let image_info = &mut self.vk_descriptors.image_infos[next_index as usize];
                        image_info.sampler = sampler.vk_sampler();
                        image_info.image_view = vk::ImageView::null();
                        image_info.image_layout = vk::ImageLayout::UNDEFINED;
                    } else {
                        todo!();
                    }
                    next_index += 1;
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .image_info(
                            &self.vk_descriptors.image_infos
                                [begin_index as usize..next_index as usize],
                        )
                        .build(),
                );
            }

            ShaderResourceType::ConstantBuffer
            | ShaderResourceType::StructuredBuffer
            | ShaderResourceType::RWStructuredBuffer
            | ShaderResourceType::ByteAdressBuffer
            | ShaderResourceType::RWByteAdressBuffer => {
                for update_data in update_datas {
                    if let DescriptorRef::BufferView(buffer_view) = update_data {
                        assert!(buffer_view.is_compatible_with_descriptor(descriptor));
                        let buffer_info =
                            &mut self.vk_descriptors.buffer_infos[next_index as usize];
                        buffer_info.buffer = buffer_view.buffer().vk_buffer();
                        buffer_info.offset = buffer_view.vk_offset();
                        buffer_info.range = buffer_view.vk_size();
                    } else {
                        todo!();
                    }
                    next_index += 1;
                }
                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .buffer_info(
                            &self.vk_descriptors.buffer_infos
                                [begin_index as usize..next_index as usize],
                        )
                        .build(),
                );
            }
            ShaderResourceType::Texture2D
            | ShaderResourceType::Texture2DArray
            | ShaderResourceType::Texture3D
            | ShaderResourceType::TextureCube
            | ShaderResourceType::TextureCubeArray => {
                for update_data in update_datas {
                    if let DescriptorRef::TextureView(texture_view) = update_data {
                        assert!(texture_view.is_compatible_with_descriptor(descriptor));
                        let image_info = &mut self.vk_descriptors.image_infos[next_index as usize];
                        next_index += 1;

                        image_info.sampler = vk::Sampler::null();
                        image_info.image_view = texture_view.vk_image_view();
                        image_info.image_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                    } else {
                        todo!();
                    }
                    next_index += 1;
                }

                self.pending_writes.push(
                    write_descriptor_builder
                        .image_info(
                            &self.vk_descriptors.image_infos
                                [begin_index as usize..next_index as usize],
                        )
                        .build(),
                );
            }
            ShaderResourceType::RWTexture2D
            | ShaderResourceType::RWTexture2DArray
            | ShaderResourceType::RWTexture3D => {
                todo!();
            }
        }

        Ok(())
    }

    fn flush(&mut self) -> GfxResult<VulkanDescriptorSetHandle> {

        if !self.pending_writes.is_empty() {
            let device = self.descriptor_set_layout.device_context().device();
            unsafe {
                device.update_descriptor_sets(&self.pending_writes, &[]);
            }

            self.pending_writes.clear();
        }

        Ok(self.descriptor_set)
    }
}
