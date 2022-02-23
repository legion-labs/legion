use lgn_graphics_api::{DescriptorHeapDef, DescriptorSet};

use crate::cgen;

use super::DescriptorHeapManager;

pub struct PersistentDescriptorSetManager {
    descriptor_set: DescriptorSet,
}

impl PersistentDescriptorSetManager {
    pub fn new(descriptor_heap_manager: &DescriptorHeapManager) -> Self {
        let layout = cgen::descriptor_set::PersistentDescriptorSet::descriptor_set_layout();

        let def = DescriptorHeapDef::from_descriptor_set_layout_def(layout.definition(), 1);

        let persistent_partition = descriptor_heap_manager
            .descriptor_heap()
            .alloc_partition(false, &def)
            .unwrap();

        let descriptor_set = persistent_partition.alloc(layout).unwrap();

        PersistentDescriptorSetManager { descriptor_set }
    }
}
