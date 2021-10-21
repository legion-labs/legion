use std::collections::BTreeMap;

use super::ResourceHandleUntyped;
use legion_data_runtime::ResourceId;

/// Mapping between a `ResourceId` and `ResourceHandleUntyped`
#[derive(Default)]
pub struct ResourceHandles(BTreeMap<ResourceId, ResourceHandleUntyped>);

impl ResourceHandles {
    /// Retrieve a `ResourceHandleUntyped` from a `ResourceId`
    pub fn get(&self, resource_id: ResourceId) -> Option<&ResourceHandleUntyped> {
        self.0.get(&resource_id)
    }

    /// Insert a `ResourceHandleUntyped`
    pub fn insert(&mut self, resource_id: ResourceId, handle: ResourceHandleUntyped) {
        self.0.insert(resource_id, handle);
    }
}
