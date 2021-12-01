use std::collections::{btree_map::IterMut, BTreeMap};

use legion_data_runtime::{ResourceId, ResourceType};

#[derive(Default)]
pub(crate) struct AssetLoadingStates(BTreeMap<(ResourceType, ResourceId), LoadingState>);

impl AssetLoadingStates {
    pub(crate) fn insert(&mut self, asset_id: (ResourceType, ResourceId), state: LoadingState) {
        self.0.insert(asset_id, state);
    }

    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, (ResourceType, ResourceId), LoadingState> {
        self.0.iter_mut()
    }

    pub(crate) fn get(&self, asset_id: (ResourceType, ResourceId)) -> Option<LoadingState> {
        self.0.get(&asset_id).copied()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum LoadingState {
    Pending,
    Loaded,
    Failed,
}
