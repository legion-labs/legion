use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSet, DescriptorSetWriter,
    DeviceContext, Sampler, TextureView,
};

use crate::{cgen, resources::IndexAllocator};

use super::DescriptorHeapManager;

const BINDLESS_TEXTURE_ARRAY_LEN: u32 = 10 * 1024;
const SAMPLER_ARRAY_SIZE: u32 = 64; // When changing this number make sure to make a corresponding change to material_samplers in root.rn

pub struct PersistentDescriptorSetManager {
    device_context: DeviceContext,
    descriptor_set: DescriptorSet,
    render_frame: u64,
    num_render_frames: u64,
    bindless_index_allocator: IndexAllocator,
    removed_indices: Vec<Vec<u32>>,

    material_textures_index: u32,
    material_samplers_index: u32,
}

impl PersistentDescriptorSetManager {
    pub fn new(
        device_context: &DeviceContext,
        descriptor_heap_manager: &DescriptorHeapManager,
        num_render_frames: u64,
    ) -> Self {
        // todo: cgen must be initialized at this point
        let layout = cgen::descriptor_set::PersistentDescriptorSet::descriptor_set_layout();

        // todo: make the size runtime-driven
        assert_eq!(
            layout
                .find_descriptor_by_name("material_textures")
                .unwrap()
                .element_count
                .get(),
            BINDLESS_TEXTURE_ARRAY_LEN
        );

        let material_textures_index = layout
            .find_descriptor_index_by_name("material_textures")
            .unwrap();
        let material_samplers_index = layout
            .find_descriptor_index_by_name("material_samplers")
            .unwrap();

        let persistent_partition = DescriptorHeapPartition::new(
            descriptor_heap_manager.descriptor_heap(),
            false,
            DescriptorHeapDef::from_descriptor_set_layout_def(layout.definition(), 1),
        );

        Self {
            device_context: device_context.clone(),
            descriptor_set: persistent_partition.alloc(layout).unwrap(),
            render_frame: 0,
            num_render_frames,
            bindless_index_allocator: IndexAllocator::new(BINDLESS_TEXTURE_ARRAY_LEN),
            removed_indices: (0..num_render_frames)
                .map(|_| Vec::new())
                .collect::<Vec<_>>(),
            material_textures_index,
            material_samplers_index,
        }
    }

    // todo: make batched versions (set_bindless_textureS with a slice?)
    pub fn set_bindless_texture(&mut self, texture_view: &TextureView) -> u32 {
        let index = self.bindless_index_allocator.acquire_index();

        let mut writer = DescriptorSetWriter::new(
            &self.device_context,
            self.descriptor_set.handle(),
            self.descriptor_set.layout(),
        );

        writer.set_descriptors_by_index_and_offset(
            self.material_textures_index,
            index,
            &[DescriptorRef::TextureView(texture_view)],
        );

        index
    }

    pub fn set_sampler(&mut self, idx: u32, sampler: &Sampler) {
        assert!(idx < SAMPLER_ARRAY_SIZE);
        let mut writer = DescriptorSetWriter::new(
            &self.device_context,
            self.descriptor_set.handle(),
            self.descriptor_set.layout(),
        );

        writer.set_descriptors_by_index_and_offset(
            self.material_samplers_index,
            idx,
            &[DescriptorRef::Sampler(sampler)],
        );
    }

    pub fn unset_bindless_texture(&mut self, index: u32) {
        let removed_indices = &mut self.removed_indices[self.render_frame as usize];
        removed_indices.push(index);
    }

    pub fn descriptor_set(&self) -> &DescriptorSet {
        &self.descriptor_set
    }

    pub fn frame_update(&mut self) {
        self.render_frame = (self.render_frame + 1) % self.num_render_frames as u64;
        self.bindless_index_allocator
            .release_indexes(&self.removed_indices[self.render_frame as usize]);
        self.removed_indices[self.render_frame as usize].clear();
    }
}
