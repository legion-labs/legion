use lgn_app::App;
use lgn_ecs::prelude::ResMut;
use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSet, DescriptorSetWriter,
    DeviceContext, TextureView,
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

        let def = DescriptorHeapDef::from_descriptor_set_layout_def(layout.definition(), 1);
        let persistent_partition =
            DescriptorHeapPartition::new(descriptor_heap_manager.descriptor_heap(), false, &def)
                .unwrap();

        Self {
            device_context: device_context.clone(),
            descriptor_set: persistent_partition.alloc(layout).unwrap(),
            render_frame: 0,
            render_frame_capacity,
            bindless_index_allocator: IndexAllocator::new(BINDLESS_TEXTURE_ARRAY_LEN),
            removed_indices: (0..render_frame_capacity)
                .map(|_| Vec::new())
                .collect::<Vec<_>>(),
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

        // todo:  cache this index
        let material_textures_index = self
            .descriptor_set
            .layout()
            .find_descriptor_index_by_name("material_textures")
            .unwrap();

        writer.set_descriptors_by_index_and_offset(
            material_textures_index,
            index,
            &[DescriptorRef::TextureView(texture_view)],
        );

        index
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
            .release_index_ids(&self.removed_indices[self.render_frame]);
        self.removed_indices[self.render_frame].clear();
    }
}

fn frame_update(mut persistent_descriptor_set_manager: ResMut<'_, PersistentDescriptorSetManager>) {
    persistent_descriptor_set_manager.frame_update();
}
