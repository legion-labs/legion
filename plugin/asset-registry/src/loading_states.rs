use std::collections::{btree_map::IterMut, BTreeMap};

use legion_data_runtime::ResourceId;

#[derive(Default)]
pub(crate) struct AssetLoadingStates(BTreeMap<ResourceId, LoadingState>);

impl AssetLoadingStates {
    pub(crate) fn insert(&mut self, asset_id: ResourceId, state: LoadingState) {
        self.0.insert(asset_id, state);
    }

    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, ResourceId, LoadingState> {
        self.0.iter_mut()
    }
}

pub(crate) enum LoadingState {
    Pending,
    Loaded,
    Failed,
}
