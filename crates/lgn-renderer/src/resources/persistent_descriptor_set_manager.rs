use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSet, DescriptorSetWriter,
    DeviceContext, TextureView,
};

use crate::cgen;

use super::DescriptorHeapManager;

pub struct PersistentDescriptorSetManager {
    device_context: DeviceContext,
    // todo: remove this option thing
    descriptor_set: Option<DescriptorSet>,
}

impl PersistentDescriptorSetManager {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
            descriptor_set: None,
        }
    }

    // todo: remove this function and initialize with cgen registry
    pub fn initialize(&mut self, descriptor_heap_manager: &DescriptorHeapManager) {
        let layout = cgen::descriptor_set::PersistentDescriptorSet::descriptor_set_layout();

        let def = DescriptorHeapDef::from_descriptor_set_layout_def(layout.definition(), 1);
        let persistent_partition =
            DescriptorHeapPartition::new(descriptor_heap_manager.descriptor_heap(), false, &def)
                .unwrap();

        self.descriptor_set = Some(persistent_partition.alloc(layout).unwrap());
    }

    pub fn set_texture_(&mut self, index: u32, texture_view: &TextureView) {
        let descriptor_set = self.descriptor_set.as_ref().unwrap();

        let mut writer = DescriptorSetWriter::new(
            &self.device_context,
            descriptor_set.handle(),
            descriptor_set.layout(),
        );

        // cache this index
        let material_textures_index = descriptor_set
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
        self.descriptor_set.as_ref().unwrap()
    }
}
