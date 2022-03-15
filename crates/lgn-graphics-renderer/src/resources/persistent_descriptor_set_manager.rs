use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSet, DescriptorSetWriter,
    DeviceContext, TextureView,
};

use crate::{cgen, resources::IndexAllocator};

use super::DescriptorHeapManager;

const BINDLESS_TEXTURE_ARRAY_LEN: u32 = 10 * 1024;

pub struct PersistentDescriptorSetManager {
    device_context: DeviceContext,
    descriptor_set: DescriptorSet,
    bindless_index_allocator: IndexAllocator,
}

impl PersistentDescriptorSetManager {
    pub fn new(
        device_context: &DeviceContext,
        descriptor_heap_manager: &DescriptorHeapManager,
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
            bindless_index_allocator: IndexAllocator::new(BINDLESS_TEXTURE_ARRAY_LEN),
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

    pub fn descriptor_set(&self) -> &DescriptorSet {
        &self.descriptor_set
    }
}
