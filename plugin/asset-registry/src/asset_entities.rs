use std::collections::BTreeMap;

use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::Entity;

#[derive(Default)]
pub(crate) struct AssetToEntityMap(BTreeMap<ResourceTypeAndId, Entity>);

impl AssetToEntityMap {
    pub(crate) fn get(&self, asset_id: ResourceTypeAndId) -> Option<Entity> {
        self.0.get(&asset_id).copied()
    }

    pub(crate) fn insert(&mut self, asset_id: ResourceTypeAndId, entity: Entity) -> Option<Entity> {
        self.0.insert(asset_id, entity)
    }
}
