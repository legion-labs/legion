use std::collections::BTreeMap;

use legion_data_runtime::{HandleUntyped, ResourceId, ResourceType};

#[derive(Default)]
pub(crate) struct AssetHandles(BTreeMap<(ResourceType, ResourceId), HandleUntyped>);

impl AssetHandles {
    pub(crate) fn get(&self, asset_id: (ResourceType, ResourceId)) -> Option<&HandleUntyped> {
        self.0.get(&asset_id)
    }

    pub(crate) fn insert(&mut self, asset_id: (ResourceType, ResourceId), handle: HandleUntyped) {
        self.0.insert(asset_id, handle);
    }
}
