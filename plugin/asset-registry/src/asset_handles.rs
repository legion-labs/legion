use std::collections::BTreeMap;

use lgn_data_runtime::{HandleUntyped, ResourceTypeAndId};

#[derive(Default)]
pub(crate) struct AssetHandles(BTreeMap<ResourceTypeAndId, HandleUntyped>);

impl AssetHandles {
    pub(crate) fn get(&self, asset_id: ResourceTypeAndId) -> Option<&HandleUntyped> {
        self.0.get(&asset_id)
    }

    pub(crate) fn insert(&mut self, asset_id: ResourceTypeAndId, handle: HandleUntyped) {
        self.0.insert(asset_id, handle);
    }
}
