use ash::vk;
use smallvec::SmallVec;

use crate::{
    DescriptorRef, DescriptorSetLayout, DescriptorSetWriter, GfxResult, ShaderResourceType,
    MAX_DESCRIPTOR_BINDINGS,
};

const BUFFER_INFOS_STACK_CAPACITY: usize = 512;
const IMAGE_INFOS_STACK_CAPACITY: usize = 512;

pub struct VulkanDescriptorSetWriter;

impl VulkanDescriptorSetWriter {
    pub fn new(descriptor_set_layout: &DescriptorSetLayout) -> GfxResult<Self> {
        if descriptor_set_layout.vk_layout() == vk::DescriptorSetLayout::null() {
            return Err("Invalid vulkan DescriptorSetLayout".into());
        }

        Ok(Self)
    }
}

impl<'a> DescriptorSetWriter<'a> {
    #[allow(clippy::unused_self, clippy::todo)]
    pub(crate) fn backend_set_descriptors_by_index_and_offset(
        &mut self,
        descriptor_index: u32,
        descriptor_offset: u32,
        descriptor_refs: &[DescriptorRef<'_>],
    ) {
        let vk_resource_info_count = descriptor_refs.len();

        let mut vk_image_infos =
            SmallVec::<[vk::DescriptorImageInfo; IMAGE_INFOS_STACK_CAPACITY]>::with_capacity(
                vk_resource_info_count,
            );
        unsafe { vk_image_infos.set_len(vk_resource_info_count) };

        let mut vk_buffer_infos =
            SmallVec::<[vk::DescriptorBufferInfo; BUFFER_INFOS_STACK_CAPACITY]>::with_capacity(
                vk_resource_info_count,
            );
        unsafe { vk_buffer_infos.set_len(vk_resource_info_count) };

        let vk_pending_writes = self.build_vk_write_descriptor_set(
            descriptor_index,
            descriptor_offset,
            descriptor_refs,
            &mut vk_image_infos,
            &mut vk_buffer_infos,
        );

        unsafe {
            self.device_context
                .vk_device()
                .update_descriptor_sets(&[vk_pending_writes], &[]);
        }
    }

    pub(crate) fn backend_set_descriptors(&mut self, descriptor_refs: &[DescriptorRef<'_>]) {
        let descriptor_count = self.descriptor_set_layout.descriptor_count();
        let vk_resource_info_count = self.descriptor_set_layout.flat_descriptor_count() as usize;

        let mut vk_pending_writes =
            SmallVec::<[vk::WriteDescriptorSet; MAX_DESCRIPTOR_BINDINGS]>::with_capacity(
                descriptor_count as usize,
            );
        unsafe { vk_pending_writes.set_len(descriptor_count as usize) };

        let mut vk_image_infos =
            SmallVec::<[vk::DescriptorImageInfo; IMAGE_INFOS_STACK_CAPACITY]>::with_capacity(
                vk_resource_info_count,
            );
        unsafe { vk_image_infos.set_len(vk_resource_info_count) };

        let mut vk_buffer_infos =
            SmallVec::<[vk::DescriptorBufferInfo; BUFFER_INFOS_STACK_CAPACITY]>::with_capacity(
                vk_resource_info_count,
            );
        unsafe { vk_buffer_infos.set_len(vk_resource_info_count) };

        for descriptor_index in 0..descriptor_count {
            let descriptor = self.descriptor_set_layout.descriptor(descriptor_index);
            let element_count = descriptor.element_count.get() as usize;
            let descriptor_refs = &descriptor_refs
                [descriptor.flat_index as usize..descriptor.flat_index as usize + element_count];
            let vk_typed_flat_index =
                self.descriptor_set_layout
                    .vk_typed_flat_index(descriptor_index) as usize;

            vk_pending_writes[descriptor_index as usize] = self.build_vk_write_descriptor_set(
                descriptor_index,
                0,
                descriptor_refs,
                &mut vk_image_infos[vk_typed_flat_index..vk_typed_flat_index + element_count],
                &mut vk_buffer_infos[vk_typed_flat_index..vk_typed_flat_index + element_count],
            );
        }

        unsafe {
            self.device_context
                .vk_device()
                .update_descriptor_sets(&vk_pending_writes, &[]);
        }
    }

    fn build_vk_write_descriptor_set(
        &mut self,
        descriptor_index: u32,
        descriptor_offset: u32,
        descriptor_refs: &[DescriptorRef<'_>],
        vk_image_infos: &mut [vk::DescriptorImageInfo],
        vk_buffer_infos: &mut [vk::DescriptorBufferInfo],
    ) -> vk::WriteDescriptorSet {
        let descriptor_set = self.descriptor_set;
        let descriptor = self.descriptor_set_layout.descriptor(descriptor_index);

        let vk_descriptor_type = super::internal::shader_resource_type_to_descriptor_type(
            descriptor.shader_resource_type,
        );

        let mut write_descriptor_builder = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set.backend_descriptor_set_handle)
            .dst_binding(descriptor_index)
            .dst_array_element(descriptor_offset)
            .descriptor_type(vk_descriptor_type);

        match descriptor.shader_resource_type {
            ShaderResourceType::Sampler => {
                for (index, descriptor_ref) in descriptor_refs.iter().enumerate() {
                    if let DescriptorRef::Sampler(sampler) = descriptor_ref {
                        let image_info = &mut vk_image_infos[index];
                        image_info.sampler = sampler.vk_sampler();
                        image_info.image_view = vk::ImageView::null();
                        image_info.image_layout = vk::ImageLayout::UNDEFINED;
                    } else {
                        unreachable!();
                    }
                }

                // Queue a descriptor write
                write_descriptor_builder = write_descriptor_builder.image_info(vk_image_infos);
            }

            ShaderResourceType::ConstantBuffer
            | ShaderResourceType::StructuredBuffer
            | ShaderResourceType::RWStructuredBuffer
            | ShaderResourceType::ByteAddressBuffer
            | ShaderResourceType::RWByteAddressBuffer => {
                for (index, descriptor_ref) in descriptor_refs.iter().enumerate() {
                    if let DescriptorRef::BufferView(buffer_view) = descriptor_ref {
                        assert!(buffer_view.is_compatible_with_descriptor(descriptor));
                        let buffer_info = &mut vk_buffer_infos[index];
                        buffer_info.buffer = buffer_view.buffer().vk_buffer();
                        buffer_info.offset = buffer_view.offset();
                        buffer_info.range = buffer_view.size();
                    } else {
                        unreachable!();
                    }
                }
                // Queue a descriptor write
                write_descriptor_builder = write_descriptor_builder.buffer_info(vk_buffer_infos);
            }
            ShaderResourceType::Texture2D
            | ShaderResourceType::Texture2DArray
            | ShaderResourceType::Texture3D
            | ShaderResourceType::TextureCube
            | ShaderResourceType::TextureCubeArray => {
                for (index, descriptor_ref) in descriptor_refs.iter().enumerate() {
                    if let DescriptorRef::TextureView(texture_view) = descriptor_ref {
                        assert!(texture_view.is_compatible_with_descriptor(descriptor));
                        let image_info = &mut vk_image_infos[index];

                        image_info.sampler = vk::Sampler::null();
                        image_info.image_view = texture_view.vk_image_view();
                        image_info.image_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                    } else {
                        unreachable!();
                    }
                }

                write_descriptor_builder = write_descriptor_builder.image_info(vk_image_infos);
            }
            ShaderResourceType::RWTexture2D
            | ShaderResourceType::RWTexture2DArray
            | ShaderResourceType::RWTexture3D => {
                for (index, descriptor_ref) in descriptor_refs.iter().enumerate() {
                    if let DescriptorRef::TextureView(texture_view) = descriptor_ref {
                        assert!(texture_view.is_compatible_with_descriptor(descriptor));
                        let image_info = &mut vk_image_infos[index];

                        image_info.sampler = vk::Sampler::null();
                        image_info.image_view = texture_view.vk_image_view();
                        image_info.image_layout = vk::ImageLayout::GENERAL;
                    } else {
                        unreachable!();
                    }
                }

                write_descriptor_builder = write_descriptor_builder.image_info(vk_image_infos);
            }
        }
        write_descriptor_builder.build()
    }
}
