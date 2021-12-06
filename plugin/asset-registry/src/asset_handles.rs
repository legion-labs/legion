use std::collections::BTreeMap;

use lgn_data_runtime::{HandleUntyped, ResourceId};

#[derive(Default)]
pub(crate) struct AssetHandles(BTreeMap<ResourceId, HandleUntyped>);

impl AssetHandles {
    pub(crate) fn get(&self, asset_id: ResourceId) -> Option<&HandleUntyped> {
        self.0.get(&asset_id)
    }

    pub(crate) fn insert(&mut self, asset_id: ResourceId, handle: HandleUntyped) {
        self.0.insert(asset_id, handle);
    }
}
