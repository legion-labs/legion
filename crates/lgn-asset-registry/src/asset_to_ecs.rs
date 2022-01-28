use std::sync::Arc;

use lgn_core::Name;
use lgn_data_runtime::{AssetRegistry, HandleUntyped, Resource, ResourceTypeAndId};
use lgn_ecs::prelude::*;
use lgn_renderer::{
    components::{RotationComponent, StaticMesh},
    resources::{DefaultMaterialType, DefaultMeshes},
};
use lgn_tracing::info;
use lgn_transform::prelude::*;
use sample_data::runtime as runtime_data;

use crate::asset_entities::AssetToEntityMap;

pub(crate) fn load_ecs_asset<T>(
    asset_id: &ResourceTypeAndId,
    handle: &HandleUntyped,
    registry: &Res<'_, Arc<AssetRegistry>>,
    commands: &mut Commands<'_, '_>,
    asset_to_entity_map: &mut ResMut<'_, AssetToEntityMap>,
    default_meshes: &Res<'_, DefaultMeshes>,
) -> bool
where
    T: AssetToECS + Resource + 'static,
{
    if asset_id.kind == T::TYPE {
        if let Some(asset) = handle.get::<T>(registry) {
            let entity = T::create_in_ecs(
                commands,
                &asset,
                asset_id,
                registry,
                asset_to_entity_map,
                default_meshes,
            );

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

pub(crate) trait AssetToECS {
    fn create_in_ecs(
        _commands: &mut Commands<'_, '_>,
        _asset: &Self,
        _asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
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
        asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
        default_meshes: &Res<'_, DefaultMeshes>,
    ) -> Option<Entity> {
        let mut entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };

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
                entity.insert(StaticMesh::from_default_meshes(
                    default_meshes,
                    static_mesh.mesh_id,
                    static_mesh.color,
                    DefaultMaterialType::Default,
                ));
            } else if let Some(script) =
                component.downcast_ref::<lgn_scripting::runtime::ScriptComponent>()
            {
                entity.insert(script.clone());
                /*if script.script.is_none() {
                    continue;
                }
                let payload = match script.script_type {
                    runtime_data::ScriptType::Mun => {
                        ECSScriptPayload::LibPath(match &script.payload {
                            runtime_data::ScriptPayload::LibPath(path) => path.clone(),
                            _ => panic!("Bad payload for LibPath!"),
                        })
                    }
                    runtime_data::ScriptType::Rune | runtime_data::ScriptType::Rhai => {
                        let script_handle: Handle<runtime_data::Script> = registry
                            .get_untyped(script.script.as_ref().unwrap().id())
                            .unwrap()
                            .into();
                        let script_ref = script_handle.get(registry);
                        ECSScriptPayload::ContainedScript(
                            std::str::from_utf8(&script_ref.unwrap().data)
                                .unwrap()
                                .to_string(),
                        )
                    }
                };
                #[allow(unsafe_code)]
                entity.insert(ECSScriptComponent {
                    script_type: unsafe { std::mem::transmute(script.script_type.clone()) },
                    input_values: script.input_values.clone(),
                    entry_fn: script.entry_fn.clone(),
                    payload,
                });*/
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
            } else if let Some(physics) = component.downcast_ref::<runtime_data::Physics>() {
                entity.insert(physics.clone());
            }
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
        _default_meshes: &Res<'_, DefaultMeshes>,
    ) -> Option<Entity> {
        let entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };
        Some(entity.id())
    }
}

impl AssetToECS for lgn_graphics_data::runtime::Material {}

impl AssetToECS for runtime_data::Mesh {}

impl AssetToECS for lgn_graphics_data::runtime_texture::Texture {}

impl AssetToECS for generic_data::runtime::DebugCube {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        instance: &Self,
        asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
        default_meshes: &Res<'_, DefaultMeshes>,
    ) -> Option<Entity> {
        let mut entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
            commands.entity(entity)
        } else {
            commands.spawn()
        };

        if !instance.name.is_empty() {
            entity.insert(Name::new(instance.name.clone()));
        }
        entity.insert(Transform {
            translation: instance.position,
            rotation: instance.rotation,
            scale: instance.scale,
        });
        entity.insert(GlobalTransform::default());
        entity.insert(StaticMesh::from_default_meshes(
            default_meshes,
            instance.mesh_id,
            instance.color,
            DefaultMaterialType::Default,
        ));
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

impl AssetToECS for lgn_scripting::runtime::Script {
    fn create_in_ecs(
        commands: &mut Commands<'_, '_>,
        entity: &Self,
        asset_id: &ResourceTypeAndId,
        _registry: &Res<'_, Arc<AssetRegistry>>,
        asset_to_entity_map: &ResMut<'_, AssetToEntityMap>,
        _default_meshes: &Res<'_, DefaultMeshes>,
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
