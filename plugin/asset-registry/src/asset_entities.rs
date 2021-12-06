use std::collections::BTreeMap;

use lgn_data_runtime::{ResourceId, ResourceType};
use lgn_ecs::prelude::Entity;

#[derive(Default)]
pub(crate) struct AssetToEntityMap(BTreeMap<(ResourceType, ResourceId), Entity>);

impl AssetToEntityMap {
    pub(crate) fn get(&self, asset_id: (ResourceType, ResourceId)) -> Option<Entity> {
        self.0.get(&asset_id).copied()
    }

    pub(crate) fn insert(
        &mut self,
        asset_id: (ResourceType, ResourceId),
        entity: Entity,
    ) -> Option<Entity> {
        self.0.insert(asset_id, entity)
    }
}
