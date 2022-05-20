use lgn_app::App;
use lgn_ecs::prelude::ResMut;
use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSet, DescriptorSetWriter,
    DeviceContext, Sampler, SamplerDef, TextureView,
};

use crate::{cgen, labels::RenderStage, resources::IndexAllocator};

use super::DescriptorHeapManager;

const BINDLESS_TEXTURE_ARRAY_LEN: u32 = 10 * 1024;

pub struct PersistentDescriptorSetManager {
    device_context: DeviceContext,
    descriptor_set: DescriptorSet,
    render_frame: usize,
    render_frame_capacity: usize,
    bindless_index_allocator: IndexAllocator,
    removed_indices: Vec<Vec<u32>>,
    samplers: Vec<Sampler>,

    material_textures_index: u32,
    material_samplers_index: u32,
}

impl PersistentDescriptorSetManager {
    pub fn new(
        device_context: &DeviceContext,
        descriptor_heap_manager: &DescriptorHeapManager,
        render_frame_capacity: usize,
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

        let def = DescriptorHeapDef::from_descriptor_set_layout_def(layout.definition(), 1);
        let persistent_partition =
            DescriptorHeapPartition::new(descriptor_heap_manager.descriptor_heap(), false, &def)
                .unwrap();

        let descriptor_set = persistent_partition.alloc(layout).unwrap();
        let mut writer = DescriptorSetWriter::new(
            device_context,
            descriptor_set.handle(),
            descriptor_set.layout(),
        );

        let sampler_def = SamplerDef::default();
        let mut samplers = Vec::new();
        let mut descriptor_refs = Vec::new();
        for _ in 0..64 {
            samplers.push(device_context.create_sampler(&sampler_def));
        }
        for sampler in &samplers {
            descriptor_refs.push(DescriptorRef::Sampler(sampler));
        }
        writer.set_descriptors_by_index(
            material_samplers_index,
            &descriptor_refs, //[DescriptorRef::Sampler(&device_context.create_sampler(&sampler_def)); 64],
        );

        Self {
            device_context: device_context.clone(),
            descriptor_set,
            render_frame: 0,
            render_frame_capacity,
            bindless_index_allocator: IndexAllocator::new(BINDLESS_TEXTURE_ARRAY_LEN),
            removed_indices: (0..render_frame_capacity)
                .map(|_| Vec::new())
                .collect::<Vec<_>>(),
            samplers,
            material_textures_index,
            material_samplers_index,
        }
    }

    pub fn init_ecs(app: &mut App) {
        app.add_system_to_stage(RenderStage::Prepare, frame_update);
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

    pub fn set_sampler(&mut self, idx: u32, sampler_def: SamplerDef) {
        let mut writer = DescriptorSetWriter::new(
            &self.device_context,
            self.descriptor_set.handle(),
            self.descriptor_set.layout(),
        );

        writer.set_descriptors_by_index_and_offset(
            self.material_samplers_index,
            idx,
            &[DescriptorRef::Sampler(
                &self.device_context.create_sampler(&sampler_def),
            )],
        );
    }

    pub fn unset_bindless_texture(&mut self, index: u32) {
        let removed_indices = &mut self.removed_indices[self.render_frame];
        removed_indices.push(index);
    }

    pub fn descriptor_set(&self) -> &DescriptorSet {
        &self.descriptor_set
    }

    fn frame_update(&mut self) {
        self.render_frame = (self.render_frame + 1) % self.render_frame_capacity;
        self.bindless_index_allocator
            .release_indexes(&self.removed_indices[self.render_frame]);
        self.removed_indices[self.render_frame].clear();
    }
}

fn frame_update(mut persistent_descriptor_set_manager: ResMut<'_, PersistentDescriptorSetManager>) {
    persistent_descriptor_set_manager.frame_update();
}
