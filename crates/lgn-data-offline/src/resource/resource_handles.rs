use std::collections::{btree_map::Entry, BTreeMap};

use lgn_data_runtime::{HandleUntyped, ResourceTypeAndId};

/// Mapping between a `ResourceId` and `HandleUntyped`
#[derive(Default)]
pub struct ResourceHandles(BTreeMap<ResourceTypeAndId, HandleUntyped>);

impl ResourceHandles {
    /// Retrieve a `HandleUntyped` from a `ResourceId`
    pub fn get(&self, resource_id: ResourceTypeAndId) -> Option<&HandleUntyped> {
        self.0.get(&resource_id)
    }

    /// Retrieve the internal hashmap entry for a `ResourceId`
    pub fn entry(
        &mut self,
        resource_id: ResourceTypeAndId,
    ) -> Entry<'_, ResourceTypeAndId, HandleUntyped> {
        self.0.entry(resource_id)
    }

    /// Insert a `HandleUntyped`
    pub fn insert(&mut self, resource_id: ResourceTypeAndId, handle: HandleUntyped) {
        self.0.insert(resource_id, handle);
    }

    /// Remove a `HandleUntyped`
    pub fn remove(&mut self, resource_id: ResourceTypeAndId) -> Option<HandleUntyped> {
        self.0.remove(&resource_id)
    }

    /// Return the Numbers of Resources
    pub fn resource_count(&self) -> usize {
        self.0.len()
    }
}
