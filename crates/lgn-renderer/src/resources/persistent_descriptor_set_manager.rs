use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSet, DescriptorSetWriter,
    DeviceContext, TextureView,
};

use crate::cgen;

use super::DescriptorHeapManager;

pub struct PersistentDescriptorSetManager {
    device_context: DeviceContext,
    descriptor_set: DescriptorSet,
}

impl PersistentDescriptorSetManager {
    pub fn new(
        device_context: &DeviceContext,
        descriptor_heap_manager: &DescriptorHeapManager,
    ) -> Self {
        // todo: cgen must be initialized at this point
        let layout = cgen::descriptor_set::PersistentDescriptorSet::descriptor_set_layout();

        let def = DescriptorHeapDef::from_descriptor_set_layout_def(layout.definition(), 1);
        let persistent_partition =
            DescriptorHeapPartition::new(descriptor_heap_manager.descriptor_heap(), false, &def)
                .unwrap();

        Self {
            device_context: device_context.clone(),
            descriptor_set: persistent_partition.alloc(layout).unwrap(),
        }
    }

    pub fn set_texture_(&mut self, index: u32, texture_view: &TextureView) {
        let mut writer = DescriptorSetWriter::new(
            &self.device_context,
            self.descriptor_set.handle(),
            self.descriptor_set.layout(),
        );

        // cache this index
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
    }

    pub fn descriptor_set(&self) -> &DescriptorSet {
        &self.descriptor_set
    }
}
