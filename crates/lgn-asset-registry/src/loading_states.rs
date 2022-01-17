use std::collections::{btree_map::IterMut, BTreeMap};

use lgn_data_runtime::ResourceTypeAndId;

#[derive(Default)]
pub(crate) struct AssetLoadingStates(BTreeMap<ResourceTypeAndId, LoadingState>);

impl AssetLoadingStates {
    pub(crate) fn insert(&mut self, asset_id: ResourceTypeAndId, state: LoadingState) {
        self.0.insert(asset_id, state);
    }

    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, ResourceTypeAndId, LoadingState> {
        self.0.iter_mut()
    }

    pub(crate) fn get(&self, asset_id: ResourceTypeAndId) -> Option<LoadingState> {
        self.0.get(&asset_id).copied()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum LoadingState {
    Pending,
    Loaded,
    Failed,
}
