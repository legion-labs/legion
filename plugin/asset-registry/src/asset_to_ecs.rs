use std::sync::Arc;

use legion_data_runtime::{AssetRegistry, HandleUntyped, Reference, Resource, ResourceId};
use legion_ecs::prelude::*;
use legion_transform::prelude::*;
use sample_data_compiler::runtime_data;

use crate::asset_entities::AssetToEntityMap;

pub(crate) fn load_ecs_asset<T>(
    asset_id: &ResourceId,
    handle: &HandleUntyped,
    registry: &ResMut<'_, Arc<AssetRegistry>>,
    commands: &mut Commands<'_, '_>,
    asset_to_entity_map: &mut ResMut<'_, AssetToEntityMap>,
) -> bool
where
    T: AssetToECS + Resource + 'static,
{
    if asset_id.ty() == T::TYPE {
        if let Some(asset) = handle.get::<T>(registry) {
            T::activate_references(asset, registry);
            let entity = T::create_in_ecs(commands, asset, asset_to_entity_map);

            if let Some(entity_id) = entity {
                asset_to_entity_map.insert(*asset_id, entity_id);

                println!(
                    "Loaded runtime asset: {} -> ECS id: {:?}",
                    asset_id, entity_id
                );
            } else {
                println!("Loaded runtime asset: {}", asset_id);
            }
        }

        true
    } else {
        false
    }
}

pub(crate) trait AssetToECS {
    fn activate_references(_asset: &Self, _registry: &ResMut<'_, Arc<AssetRegistry>>) {}

    fn create_in_ecs(
        _commands: &mut Commands<'_, '_>,
        _asset: &Self,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        None
    }
}

impl AssetToECS for runtime_data::Entity {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        runtime_entity: &Self,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        let mut entity = commands.spawn();

        let mut transform_inserted = false;
        for component in &runtime_entity.components {
            if let Some(transform) = component.downcast_ref::<runtime_data::Transform>() {
                entity.insert(Transform {
                    translation: transform.position,
                    rotation: transform.rotation,
                    scale: transform.scale,
                });
                transform_inserted = true;
            }
            //} else if let Some(visual) = component.downcast_ref::<runtime_data::Visual>() {
            // } else if let Some(gi) = component.downcast_ref::<runtime_data::GlobalIllumination>() {
            // } else if let Some(nav_mesh) = component.downcast_ref::<runtime_data::NavMesh>() {
            // } else if let Some(view) = component.downcast_ref::<runtime_data::View>() {
            // } else if let Some(light) = component.downcast_ref::<runtime_data::Light>() {
            // } else if let Some(physics) = component.downcast_ref::<runtime_data::Physics>() {
        }

        if !transform_inserted {
            entity.insert(Transform::identity());
        }
        entity.insert(GlobalTransform::identity());

        // parent, if it exists, must already be loaded since parents load their children
        let parent = if let Reference::Passive(parent) = runtime_entity.parent {
            asset_to_entity_map.get(parent)
        } else {
            None
        };

        if let Some(parent) = parent {
            entity.insert(Parent(parent));
        }

        let entity_id = entity.id();

        if let Some(parent) = parent {
            commands.entity(parent).push_children(&[entity_id]);
        }

        Some(entity_id)
    }
}

impl AssetToECS for runtime_data::Instance {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        _instance: &Self,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        let entity = commands.spawn();

        Some(entity.id())
    }
}

impl AssetToECS for legion_graphics_runtime::Material {}

impl AssetToECS for runtime_data::Mesh {}

impl AssetToECS for legion_graphics_runtime::Texture {}
