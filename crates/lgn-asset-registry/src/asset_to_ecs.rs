use std::sync::Arc;

use lgn_core::Name;
use lgn_data_runtime::{AssetRegistry, HandleUntyped, Resource, ResourceTypeAndId};
use lgn_ecs::prelude::*;
use lgn_renderer::{
    components::{RotationComponent, StaticMesh},
    resources::DefaultMeshes,
};
use lgn_scripting::components::ECSScriptComponent;
use lgn_tracing::info;
use lgn_transform::prelude::*;
use sample_data_runtime as runtime_data;

use crate::asset_entities::AssetToEntityMap;

pub(crate) fn load_ecs_asset<T>(
    asset_id: &ResourceTypeAndId,
    handle: &HandleUntyped,
    registry: &ResMut<'_, Arc<AssetRegistry>>,
    commands: &mut Commands<'_, '_>,
    asset_to_entity_map: &mut ResMut<'_, AssetToEntityMap>,
    default_meshes: &Res<'_, DefaultMeshes>,
) -> bool
where
    T: AssetToECS + Resource + 'static,
{
    if asset_id.kind == T::TYPE {
        if let Some(asset) = handle.get::<T>(registry) {
            let entity = T::create_in_ecs(commands, &asset, asset_to_entity_map, default_meshes);

            if let Some(entity_id) = entity {
                if let Some(old_entity) = asset_to_entity_map.insert(*asset_id, entity_id) {
                    commands.entity(old_entity).despawn();
                }

                info!(
                    "Loaded {}: {} -> ECS id: {:?}",
                    T::TYPENAME,
                    *asset_id,
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

pub(crate) trait AssetToECS {
    fn create_in_ecs(
        _commands: &mut Commands<'_, '_>,
        _asset: &Self,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
        _default_meshes: &Res<'_, DefaultMeshes>,
    ) -> Option<Entity> {
        None
    }
}

impl AssetToECS for runtime_data::Entity {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        runtime_entity: &Self,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
        default_meshes: &Res<'_, DefaultMeshes>,
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
            } else if let Some(static_mesh) = component.downcast_ref::<runtime_data::StaticMesh>() {
                entity.insert(StaticMesh {
                    mesh_id: static_mesh.mesh_id,
                    color: (255, 0, 0).into(),
                    vertex_offset: default_meshes.mesh_offset_from_id(static_mesh.mesh_id as u32),
                    num_vertices: default_meshes
                        .mesh_from_id(static_mesh.mesh_id as u32)
                        .num_vertices() as u32,
                    world_offset: 0,
                    picking_id: 0,
                });
            } else if let Some(script) = component.downcast_ref::<runtime_data::ScriptComponent>() {
                if script.script.is_none() {
                    continue;
                }
                entity.insert(ECSScriptComponent {
                    input_values: script.input_values.clone(),
                    entry_fn: script.entry_fn.clone(),
                    lib_path: script.lib_path.clone(),
                });
            }
            // } else if let Some(visual) = component.downcast_ref::<runtime_data::Visual>() {
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

        // parent, if it exists, must already be loaded since parents load their
        // children
        let parent = runtime_entity
            .parent
            .as_ref()
            .and_then(|parent| asset_to_entity_map.get(parent.id()));

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
        _default_meshes: &Res<'_, DefaultMeshes>,
    ) -> Option<Entity> {
        let entity = commands.spawn();

        Some(entity.id())
    }
}

impl AssetToECS for lgn_graphics_runtime::Material {}

impl AssetToECS for runtime_data::Mesh {}

impl AssetToECS for lgn_graphics_runtime::Texture {}

impl AssetToECS for generic_data::runtime::DebugCube {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        instance: &Self,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
        default_meshes: &Res<'_, DefaultMeshes>,
    ) -> Option<Entity> {
        let mut entity = commands.spawn();

        if !instance.name.is_empty() {
            entity.insert(Name::new(instance.name.clone()));
        }
        entity.insert(Transform {
            translation: instance.position,
            rotation: instance.rotation,
            scale: instance.scale,
        });
        entity.insert(StaticMesh {
            mesh_id: instance.mesh_id,
            color: instance.color,
            vertex_offset: default_meshes.mesh_offset_from_id(instance.mesh_id as u32),
            num_vertices: default_meshes
                .mesh_from_id(instance.mesh_id as u32)
                .num_vertices() as u32,
            world_offset: 0,
            picking_id: 0,
        });

        entity.insert(RotationComponent {
            rotation_speed: (
                instance.rotation_speed.x,
                instance.rotation_speed.y,
                instance.rotation_speed.z,
            ),
        });

        Some(entity.id())
    }
}

/*impl AssetToECS for runtime_data::Script {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        _instance: &Self,
        _asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
        _default_meshes: &Res<'_, DefaultMeshes>,
    ) -> Option<Entity> {
        let entity = commands.spawn();

        Some(entity.id())
    }
}*/
