use std::collections::BTreeMap;

use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::{Commands, Entity};
use lgn_tracing::info;

#[derive(Default)]
pub struct AssetToEntityMap {
    pub(crate) asset_to_entity: BTreeMap<ResourceTypeAndId, Entity>,
    pub(crate) entity_to_asset: BTreeMap<Entity, ResourceTypeAndId>,
}

impl AssetToEntityMap {
    pub fn get(&self, asset_id: ResourceTypeAndId) -> Option<Entity> {
        self.asset_to_entity.get(&asset_id).copied()
    }

    pub fn get_resource_id(&self, entity: Entity) -> Option<ResourceTypeAndId> {
        self.entity_to_asset.get(&entity).copied()
    }

    pub fn insert(&mut self, asset_id: ResourceTypeAndId, entity: Entity) -> Option<Entity> {
        let old_entity = self.asset_to_entity.insert(asset_id, entity);
        old_entity.and_then(|old_entity| self.entity_to_asset.remove(&old_entity));
        self.entity_to_asset.insert(entity, asset_id);
        old_entity
    }

    pub fn remove(&mut self, entity: Entity) {
        let old_res_id = self.entity_to_asset.remove(&entity);
        old_res_id.and_then(|old_res_id| self.asset_to_entity.remove(&old_res_id));
    }

    pub fn clear_all(&mut self, commands: &mut Commands<'_, '_>) {
        for (entity, res_id) in &self.entity_to_asset {
            info!("Unspawning {:?} -> ECS id: {}", res_id, entity.id());
            commands.entity(*entity).despawn();
        }
        self.entity_to_asset.clear();
        self.asset_to_entity.clear();
    }
}
