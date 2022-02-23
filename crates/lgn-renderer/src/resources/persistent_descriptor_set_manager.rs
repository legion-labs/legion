use lgn_graphics_api::{DescriptorHeapDef, DescriptorSet};

use crate::cgen;

use super::DescriptorHeapManager;

pub struct PersistentDescriptorSetManager {
    descriptor_set: Option<DescriptorSet>,
}

impl PersistentDescriptorSetManager {
    pub fn new() -> Self {
        Self {
            descriptor_set: None,
        }
    }

    pub fn initialize(&mut self, descriptor_heap_manager: &DescriptorHeapManager) {
        let layout = cgen::descriptor_set::PersistentDescriptorSet::descriptor_set_layout();

        let def = DescriptorHeapDef::from_descriptor_set_layout_def(layout.definition(), 1);

        let persistent_partition = descriptor_heap_manager
            .descriptor_heap()
            .alloc_partition(false, &def)
            .unwrap();

        self.descriptor_set = Some(persistent_partition.alloc(layout).unwrap());
    }
}
