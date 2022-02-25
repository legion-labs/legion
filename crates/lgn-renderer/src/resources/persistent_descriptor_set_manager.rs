use lgn_graphics_api::{DescriptorHeapDef, DescriptorHeapPartition, DescriptorSet};

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
        let persistent_partition =
            DescriptorHeapPartition::new(descriptor_heap_manager.descriptor_heap(), false, &def)
                .unwrap();

        self.descriptor_set = Some(persistent_partition.alloc(layout).unwrap());
    }

    pub fn descriptor_set(&self) -> &DescriptorSet {
        self.descriptor_set.as_ref().unwrap()
    }
}
