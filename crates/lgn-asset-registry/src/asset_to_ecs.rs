use std::sync::Arc;

use crate::asset_entities::AssetToEntityMap;
use lgn_core::Name;
use lgn_data_runtime::{AssetRegistry, HandleUntyped, Resource, ResourceTypeAndId};
use lgn_ecs::prelude::*;
use lgn_renderer::components::{MaterialComponent, TextureComponent, VisualComponent};
use lgn_tracing::info;
use lgn_transform::prelude::*;
use sample_data::runtime as runtime_data;

pub(crate) fn load_ecs_asset<T>(
    asset_id: &ResourceTypeAndId,
    handle: &HandleUntyped,
    registry: &Res<'_, Arc<AssetRegistry>>,
    commands: &mut Commands<'_, '_>,
    asset_to_entity_map: &mut ResMut<'_, AssetToEntityMap>,
) -> bool
where
    T: AssetToECS + Resource + 'static,
{
    if asset_id.kind == T::TYPE {
        if let Some(asset) = handle.get::<T>(registry) {
            let entity =
                T::create_in_ecs(commands, &asset, asset_id, registry, asset_to_entity_map);

            if let Some(entity_id) = entity {
                if let Some(old_entity) = asset_to_entity_map.insert(*asset_id, entity_id) {
                    if entity_id.to_bits() != old_entity.to_bits() {
                        commands.entity(old_entity).despawn();
                    }
                }

                info!(
                    "Loaded {}: {} -> ECS id: {:?}",
                    T::TYPENAME,
                    asset_id.id,
                    entity_id,
                );
            } else {
                info!("Loaded {}: {}", T::TYPENAME, *asset_id);
            }
        }

        true
    } else {
        false
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) trait AssetToECS {
    fn create_in_ecs(
        _commands: &mut Commands<'_, '_>,
        _asset: &Self,
        _asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        None
    }
}

impl AssetToECS for runtime_data::Entity {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        runtime_entity: &Self,
        asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        let mut entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };

        let mut transform_inserted = false;
        let mut name_inserted = false;
        for component in &runtime_entity.components {
            if let Some(transform) = component.downcast_ref::<runtime_data::Transform>() {
                entity.insert(Transform {
                    translation: transform.position,
                    rotation: transform.rotation,
                    scale: transform.scale,
                });
                transform_inserted = true;
            } else if let Some(static_mesh) = component.downcast_ref::<runtime_data::StaticMesh>() {
                entity.insert(VisualComponent::new(
                    static_mesh.mesh_id as usize,
                    static_mesh.color,
                ));
            } else if let Some(script) =
                component.downcast_ref::<lgn_scripting::runtime::ScriptComponent>()
            {
                entity.insert(script.clone());
            } else if let Some(name) = component.downcast_ref::<runtime_data::Name>() {
                name_inserted = true;
                entity.insert(Name::new(name.name.clone()));
            } else if let Some(visual) = component.downcast_ref::<runtime_data::Visual>() {
                entity.insert(visual.clone());
            } else if let Some(gi) = component.downcast_ref::<runtime_data::GlobalIllumination>() {
                entity.insert(gi.clone());
            } else if let Some(nav_mesh) = component.downcast_ref::<runtime_data::NavMesh>() {
                entity.insert(nav_mesh.clone());
            } else if let Some(view) = component.downcast_ref::<runtime_data::View>() {
                entity.insert(view.clone());
            } else if let Some(light) = component.downcast_ref::<runtime_data::Light>() {
                entity.insert(light.clone());
            } else if let Some(physics) =
                component.downcast_ref::<lgn_physics::runtime::PhysicsRigidActor>()
            {
                entity.insert(physics.clone());
            }
        }

        if !name_inserted {
            entity.insert(Name::new(asset_id.id.to_string()));
        }

        if !transform_inserted {
            entity.insert(Transform::identity());
        }
        entity.insert(GlobalTransform::identity());

        let entity_id = entity.id();

        // try to hook the parent
        if let Some(parent_id) = runtime_entity.parent.as_ref() {
            if let Some(parent) = asset_to_entity_map.get(parent_id.id()) {
                entity.insert(Parent(parent));
                entity.commands().entity(parent).push_children(&[entity_id]);
            }
        }

        // try to hook the children
        for child_ref in &runtime_entity.children {
            if let Some(child_entity) = asset_to_entity_map.get(child_ref.id()) {
                entity.push_children(&[child_entity]);
            }
        }

        Some(entity_id)
    }
}

impl AssetToECS for runtime_data::Instance {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        _instance: &Self,
        asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        let entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };
        Some(entity.id())
    }
}

impl AssetToECS for lgn_graphics_data::runtime::Material {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        material: &Self,
        asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        let mut entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };

        entity.insert(MaterialComponent::new(
            *asset_id,
            material.albedo.clone(),
            material.normal.clone(),
            material.metalness.clone(),
            material.roughness.clone(),
        ));

        Some(entity.id())
    }
}

impl AssetToECS for runtime_data::Mesh {}

impl AssetToECS for lgn_graphics_data::runtime_texture::Texture {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        texture: &Self,
        asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        let mut entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };

        let texture_component = TextureComponent::new(
            *asset_id,
            texture.width,
            texture.height,
            texture.format,
            texture // TODO: Avoid cloning in the future
                .texture_data
                .iter()
                .map(|bytebuf| bytebuf.clone().into_vec())
                .collect::<Vec<_>>(),
        );

        entity.insert(texture_component);

        Some(entity.id())
    }
}

impl AssetToECS for lgn_scripting::runtime::Script {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        entity: &Self,
        asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
    ) -> Option<Entity> {
        let ecs_entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };

        info!(
            "Loading script resource {} bytes",
            entity.compiled_script.len()
        );

        Some(ecs_entity.id())
    }
}
