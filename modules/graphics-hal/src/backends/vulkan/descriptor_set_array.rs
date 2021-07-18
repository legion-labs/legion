use super::{
    DescriptorSetLayoutInfo, VulkanApi, VulkanDescriptorHeap, VulkanDeviceContext,
    VulkanRootSignature,
};
use crate::{
    DescriptorKey, DescriptorSetArray, DescriptorSetArrayDef, DescriptorSetHandle,
    DescriptorUpdate, GfxResult, ResourceType, TextureBindType,
};
use ash::version::DeviceV1_0;
use ash::vk;

struct DescriptorUpdateData {
    // one per set * elements in each descriptor
    image_infos: Vec<vk::DescriptorImageInfo>,
    buffer_infos: Vec<vk::DescriptorBufferInfo>,
    buffer_views: Vec<vk::BufferView>,
    update_data_count: usize,
}

impl DescriptorUpdateData {
    fn new(update_data_count: usize) -> Self {
        Self {
            image_infos: vec![vk::DescriptorImageInfo::default(); update_data_count],
            buffer_infos: vec![vk::DescriptorBufferInfo::default(); update_data_count],
            buffer_views: vec![vk::BufferView::default(); update_data_count],
            update_data_count,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VulkanDescriptorSetHandle(pub vk::DescriptorSet);

impl DescriptorSetHandle<VulkanApi> for VulkanDescriptorSetHandle {}

pub struct VulkanDescriptorSetArray {
    root_signature: VulkanRootSignature,
    set_index: u32,
    // one per set
    descriptor_sets: Vec<vk::DescriptorSet>,
    //update_data: Vec<UpdateData>,
    //dynamic_size_offset: Option<SizeOffset>,
    update_data: DescriptorUpdateData,
    //WARNING: This contains pointers into data stored in DescriptorUpdateData, however those
    // vectors are not added/removed from so their addresses will remain stable, even if this
    // struct is moved
    pending_writes: Vec<vk::WriteDescriptorSet>,
}

impl std::fmt::Debug for VulkanDescriptorSetArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanDescriptorSetArray")
            .field("first_descriptor_set", &self.descriptor_sets[0])
            .field("root_signature", &self.root_signature)
            .field("set_index", &self.set_index)
            .field("pending_write_count", &self.pending_writes.len())
            .finish()
    }
}

// For *const c_void in vk::WriteDescriptorSet, which always point at contents of vectors in
// update_data that never get resized
unsafe impl Send for VulkanDescriptorSetArray {}
unsafe impl Sync for VulkanDescriptorSetArray {}

impl VulkanDescriptorSetArray {
    pub fn set_index(&self) -> u32 {
        self.set_index
    }

    pub fn vk_descriptor_set(&self, index: u32) -> Option<vk::DescriptorSet> {
        self.descriptor_sets.get(index as usize).copied()
    }

    pub(crate) fn new(
        device_context: &VulkanDeviceContext,
        heap: &VulkanDescriptorHeap,
        descriptor_set_array_def: &DescriptorSetArrayDef<'_, VulkanApi>,
    ) -> GfxResult<Self> {
        let root_signature = descriptor_set_array_def.root_signature.clone();
        let layout_index = descriptor_set_array_def.set_index as usize;
        let update_data_count = descriptor_set_array_def.array_length
            * root_signature.inner.layouts[layout_index].update_data_count_per_set as usize;
        // let dynamic_offset_count = root_signature.layouts[layout_index]
        //     .dynamic_descriptor_indexes
        //     .len();

        let descriptor_set_layout = root_signature.inner.descriptor_set_layouts[layout_index];

        // these persist
        let mut descriptors_set_layouts = Vec::with_capacity(descriptor_set_array_def.array_length);
        //let mut update_data = Vec::with_capacity(descriptor_set_array_def.array_length * update_data_count);
        let update_data = DescriptorUpdateData::new(update_data_count);

        if root_signature.inner.descriptor_set_layouts[layout_index]
            == vk::DescriptorSetLayout::null()
        {
            return Err("Descriptor set layout does not exist in this root signature".into());
        }

        for _ in 0..descriptor_set_array_def.array_length {
            descriptors_set_layouts.push(descriptor_set_layout);

            // for _ in 0..update_data_count {
            //     //TODO: copy it from root signature update template
            //     update_data.push(UpdateData::default());
            // }
        }

        let descriptor_sets =
            heap.allocate_descriptor_sets(device_context.device(), &descriptors_set_layouts)?;

        // let dynamic_size_offset = if dynamic_offset_count > 0 {
        //     assert_eq!(1, dynamic_offset_count);
        //     Some(SizeOffset)
        // } else {
        //     None
        // };

        Ok(Self {
            root_signature,
            set_index: descriptor_set_array_def.set_index,
            descriptor_sets,
            update_data,
            pending_writes: Vec::default(),
        })
    }
}

impl DescriptorSetArray<VulkanApi> for VulkanDescriptorSetArray {
    fn handle(&self, index: u32) -> Option<VulkanDescriptorSetHandle> {
        self.descriptor_sets
            .get(index as usize)
            .map(|x| VulkanDescriptorSetHandle(*x))
    }

    fn root_signature(&self) -> &VulkanRootSignature {
        &self.root_signature
    }

    fn update_descriptor_set(
        &mut self,
        descriptor_updates: &[DescriptorUpdate<'_, VulkanApi>],
    ) -> GfxResult<()> {
        for update in descriptor_updates {
            self.queue_descriptor_set_update(update)?;
        }
        self.flush_descriptor_set_updates()
    }

    fn queue_descriptor_set_update(
        &mut self,
        update: &DescriptorUpdate<'_, VulkanApi>,
    ) -> GfxResult<()> {
        let layout: &DescriptorSetLayoutInfo =
            &self.root_signature.inner.layouts[self.set_index as usize];
        let descriptor_index = match &update.descriptor_key {
            DescriptorKey::Name(name) => {
                let descriptor_index = self.root_signature.find_descriptor_by_name(name);
                if let Some(descriptor_index) = descriptor_index {
                    let set_index = self
                        .root_signature
                        .descriptor(descriptor_index)
                        .unwrap()
                        .set_index;
                    if set_index == self.set_index {
                        descriptor_index
                    } else {
                        return Err(format!(
                            "Found descriptor {:?} but it's set_index ({:?}) does not match the set ({:?})",
                            &update.descriptor_key,
                            set_index,
                            self.set_index
                        ).into());
                    }
                } else {
                    return Err(
                        format!("Could not find descriptor {:?}", &update.descriptor_key).into(),
                    );
                }
            }
            DescriptorKey::Binding(binding) => layout
                .binding_to_descriptor_index
                .get(binding)
                .copied()
                .ok_or_else(|| format!("Could not find descriptor {:?}", update.descriptor_key,))?,
            DescriptorKey::DescriptorIndex(descriptor_index) => *descriptor_index,
            DescriptorKey::Undefined => {
                return Err("Passed DescriptorKey::Undefined to update_descriptor_set()".into())
            }
        };

        //let descriptor_index = descriptor_index.ok_or_else(|| format!("Could not find descriptor {:?}", &update.descriptor_key))?;
        let descriptor = self.root_signature.descriptor(descriptor_index).unwrap();

        let descriptor_first_update_data = descriptor.update_data_offset_in_set.unwrap()
            + (layout.update_data_count_per_set * update.array_index);

        //let mut descriptor_set_writes = Vec::default();

        let vk_set = self.descriptor_sets[update.array_index as usize];
        let write_descriptor_builder = vk::WriteDescriptorSet::builder()
            .dst_set(vk_set)
            .dst_binding(descriptor.binding)
            .dst_array_element(update.dst_element_offset)
            .descriptor_type(descriptor.vk_type);

        log::trace!(
            "update descriptor set {:?} (set_index: {:?} binding: {} name: {:?} type: {:?} array_index: {} first update data index: {} set: {:?})",
            update.descriptor_key,
            descriptor.set_index,
            descriptor.binding,
            descriptor.name,
            descriptor.resource_type,
            update.array_index,
            descriptor_first_update_data,
            vk_set
        );

        match descriptor.resource_type {
            ResourceType::SAMPLER => {
                if descriptor.has_immutable_sampler {
                    return Err(format!(
                        "Tried to update sampler {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but it is a static/immutable sampler",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    ).into());
                }

                let samplers = update.elements.samplers.ok_or_else(||
                    format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the samplers element list was None",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    )
                )?;
                let begin_index =
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + samplers.len() <= self.update_data.update_data_count);

                // Modify the update data
                let mut next_index = begin_index;
                for sampler in samplers {
                    let image_info = &mut self.update_data.image_infos[next_index];
                    next_index += 1;

                    image_info.sampler = sampler.vk_sampler();
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .image_info(&self.update_data.image_infos[begin_index..next_index])
                        .build(),
                );
            }
            ResourceType::COMBINED_IMAGE_SAMPLER => {
                if !descriptor.has_immutable_sampler {
                    return Err(format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the sampler is NOT immutable. This is not currently supported.",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type
                    ).into());
                }

                let textures = update.elements.textures.ok_or_else(||
                    format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the texture element list was None",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    )
                )?;
                let begin_index =
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + textures.len() <= self.update_data.update_data_count);

                let texture_bind_type = update.texture_bind_type.unwrap_or(TextureBindType::Srv);

                // Modify the update data
                let mut next_index = begin_index;
                for texture in textures {
                    let image_info = &mut self.update_data.image_infos[next_index];
                    next_index += 1;

                    if texture_bind_type == TextureBindType::SrvStencil {
                        image_info.image_view = texture.vk_srv_view_stencil().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) as TextureBindType::SrvStencil but there is no srv_stencil view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    } else if texture_bind_type == TextureBindType::Srv {
                        image_info.image_view = texture.vk_srv_view().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) as TextureBindType::Srv but there is no srv_stencil view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    } else {
                        return Err(format!(
                            "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but texture_bind_type {:?} was unexpected for this kind of resource",
                            update.descriptor_key,
                            descriptor.set_index,
                            descriptor.binding,
                            descriptor.name,
                            descriptor.resource_type,
                            update.texture_bind_type
                        ).into());
                    }

                    image_info.image_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .image_info(&self.update_data.image_infos[begin_index..next_index])
                        .build(),
                );
            }
            ResourceType::TEXTURE => {
                let textures = update.elements.textures.ok_or_else(||
                    format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the texture element list was None",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    )
                )?;
                let begin_index =
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + textures.len() <= self.update_data.update_data_count);

                let texture_bind_type = update.texture_bind_type.unwrap_or(TextureBindType::Srv);

                // Modify the update data
                let mut next_index = begin_index;
                for texture in textures {
                    let image_info = &mut self.update_data.image_infos[next_index];
                    next_index += 1;

                    if texture_bind_type == TextureBindType::SrvStencil {
                        image_info.image_view = texture.vk_srv_view_stencil().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) as TextureBindType::SrvStencil but there is no srv_stencil view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    } else if texture_bind_type == TextureBindType::Srv {
                        image_info.image_view = texture.vk_srv_view().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) as TextureBindType::Srv but there is no srv view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    } else {
                        return Err(format!(
                            "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but texture_bind_type {:?} was unexpected for this kind of resource",
                            update.descriptor_key,
                            descriptor.set_index,
                            descriptor.binding,
                            descriptor.name,
                            descriptor.resource_type,
                            update.texture_bind_type
                        ).into());
                    }

                    image_info.image_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .image_info(&self.update_data.image_infos[begin_index..next_index])
                        .build(),
                );
            }
            ResourceType::TEXTURE_READ_WRITE => {
                let textures = update.elements.textures.ok_or_else(||
                    format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the texture element list was None",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    )
                )?;
                let begin_index =
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + textures.len() <= self.update_data.update_data_count);

                // Modify the update data
                let mut next_index = begin_index;

                let texture_bind_type = update
                    .texture_bind_type
                    .unwrap_or(TextureBindType::UavMipSlice(0));

                if let TextureBindType::UavMipSlice(slice) = texture_bind_type {
                    for texture in textures {
                        let image_info = &mut self.update_data.image_infos[next_index];
                        next_index += 1;

                        let image_views = texture.vk_uav_views();
                        let image_view = *image_views.get(slice as usize).ok_or_else(|| format!(
                            "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the chosen mip slice {} exceeds the mip count of {} in the image",
                            update.descriptor_key,
                            descriptor.set_index,
                            descriptor.binding,
                            descriptor.name,
                            descriptor.resource_type,
                            slice,
                            image_views.len()
                        ))?;
                        image_info.image_view = image_view;

                        image_info.image_layout = vk::ImageLayout::GENERAL;
                    }
                } else if texture_bind_type == TextureBindType::UavMipChain {
                    let texture = textures.first().unwrap();

                    let image_views = texture.vk_uav_views();
                    if image_views.len() > descriptor.element_count as usize {
                        return Err(format!(
                            "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) using UavMipChain but the mip chain has {} images and the descriptor has {} elements",
                            update.descriptor_key,
                            descriptor.set_index,
                            descriptor.binding,
                            descriptor.name,
                            descriptor.resource_type,
                            image_views.len(),
                            descriptor.element_count
                        ).into());
                    }

                    for image_view in image_views {
                        let image_info = &mut self.update_data.image_infos[next_index];
                        next_index += 1;

                        image_info.image_view = *image_view;
                        image_info.image_layout = vk::ImageLayout::GENERAL;
                    }
                } else {
                    return Err(format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but texture_bind_type {:?} was unexpected for this kind of resource",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                        update.texture_bind_type
                    ).into());
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .image_info(&self.update_data.image_infos[begin_index..next_index])
                        .build(),
                );
            }
            ResourceType::UNIFORM_BUFFER
            | ResourceType::BUFFER
            | ResourceType::BUFFER_READ_WRITE => {
                if descriptor.vk_type == vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC {
                    //TODO: Add support for dynamic uniforms
                    unreachable!();
                }

                let buffers = update.elements.buffers.ok_or_else(||
                    format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the buffers element list was None",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    )
                )?;
                let begin_index =
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + buffers.len() <= self.update_data.update_data_count);

                // Modify the update data
                let mut next_index = begin_index;
                for (buffer_index, buffer) in buffers.iter().enumerate() {
                    let buffer_info = &mut self.update_data.buffer_infos[next_index];
                    next_index += 1;

                    buffer_info.buffer = buffer.vk_buffer();
                    buffer_info.offset = 0;
                    buffer_info.range = vk::WHOLE_SIZE;

                    if let Some(offset_size) = update.elements.buffer_offset_sizes {
                        if offset_size[buffer_index].byte_offset != 0 {
                            buffer_info.offset = offset_size[buffer_index].byte_offset;
                        }

                        if offset_size[buffer_index].size != 0 {
                            buffer_info.range = offset_size[buffer_index].size;
                        }
                    }
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .buffer_info(&self.update_data.buffer_infos[begin_index..next_index])
                        .build(),
                );
            }
            ResourceType::TEXEL_BUFFER | ResourceType::TEXEL_BUFFER_READ_WRITE => {
                let buffers = update.elements.buffers.ok_or_else(||
                    format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the buffers element list was None",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    )
                )?;
                let begin_index =
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + buffers.len() <= self.update_data.update_data_count);

                // Modify the update data
                let mut next_index = begin_index;
                for buffer in buffers {
                    let buffer_view = &mut self.update_data.buffer_views[next_index];
                    next_index += 1;

                    if descriptor.resource_type == ResourceType::TEXEL_BUFFER {
                        *buffer_view = buffer.vk_uniform_texel_view().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but there was no uniform texel view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    } else {
                        *buffer_view = buffer.vk_storage_texel_view().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but there was no storage texel view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    };
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .texel_buffer_view(&self.update_data.buffer_views[begin_index..next_index])
                        .build(),
                );
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    fn flush_descriptor_set_updates(&mut self) -> GfxResult<()> {
        if !self.pending_writes.is_empty() {
            let device = self.root_signature.device_context().device();
            unsafe {
                device.update_descriptor_sets(&self.pending_writes, &[]);
            }

            self.pending_writes.clear();
        }

        Ok(())
    }
}
