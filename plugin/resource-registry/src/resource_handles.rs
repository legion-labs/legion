use std::collections::BTreeMap;

use legion_data_offline::resource::ResourceHandleUntyped;
use legion_data_runtime::ResourceId;

#[derive(Default)]
pub(crate) struct ResourceHandles(BTreeMap<ResourceId, ResourceHandleUntyped>);

impl ResourceHandles {
    pub(crate) fn get(&self, resource_id: ResourceId) -> Option<&ResourceHandleUntyped> {
        self.0.get(&resource_id)
    }

    pub(crate) fn insert(&mut self, resource_id: ResourceId, handle: ResourceHandleUntyped) {
        self.0.insert(resource_id, handle);
    }
}
