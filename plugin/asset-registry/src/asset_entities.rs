use std::collections::BTreeMap;

use legion_data_runtime::ResourceId;
use legion_ecs::prelude::Entity;

#[derive(Default)]
pub(crate) struct AssetToEntityMap(BTreeMap<ResourceId, Entity>);

impl AssetToEntityMap {
    pub(crate) fn get(&self, asset_id: ResourceId) -> Option<Entity> {
        self.0.get(&asset_id).copied()
    }

    pub(crate) fn insert(&mut self, asset_id: ResourceId, entity: Entity) {
        self.0.insert(asset_id, entity);
    }
}
