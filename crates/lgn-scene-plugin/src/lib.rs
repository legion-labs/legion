//! Scene/Entity manager

// crate-specific lint exceptions:
//#![allow()]

use std::{collections::HashMap, sync::Arc};

use lgn_app::{App, Plugin};
use lgn_data_runtime::{AssetRegistry, AssetRegistryEvent, Resource, ResourceTypeAndId};

use lgn_ecs::prelude::*;

mod scene_instance;
use lgn_hierarchy::Children;
use scene_instance::SceneInstance;

/// Message Scene Management
pub enum SceneMessage {
    OpenScene(ResourceTypeAndId),
    CloseScene(ResourceTypeAndId),
}

pub type ActiveScenes = HashMap<ResourceTypeAndId, SceneInstance>;

#[derive(Default)]
pub struct ScenePlugin {}

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::process_load_events)
            .add_system(Self::handle_scene_messages)
            .insert_resource(ActiveScenes::default())
            .add_event::<SceneMessage>();
    }
}

impl ScenePlugin {
    #[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
    fn handle_scene_messages(
        asset_registry: Res<'_, Arc<AssetRegistry>>,
        mut scene_events: EventReader<'_, '_, SceneMessage>,
        mut active_scenes: ResMut<'_, ActiveScenes>,
        mut commands: Commands<'_, '_>,
        entity_with_children_query: Query<'_, '_, &Children>,
    ) {
        for event in scene_events.iter() {
            match event {
                SceneMessage::OpenScene(resource_id) => {
                    let scene = active_scenes
                        .entry(*resource_id)
                        .or_insert_with(|| SceneInstance::new(*resource_id));

                    scene.spawn_entity_hierarchy(
                        *resource_id,
                        &asset_registry,
                        &mut commands,
                        &entity_with_children_query,
                    );
                }
                SceneMessage::CloseScene(resource_id) => {
                    if let Some(mut scene) = active_scenes.remove(resource_id) {
                        scene.asset_to_entity_map.clear_all(&mut commands);
                    }
                }
            }
        }
    }

    #[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
    fn process_load_events(
        asset_registry: Res<'_, Arc<AssetRegistry>>,
        mut asset_loaded_events: EventReader<'_, '_, AssetRegistryEvent>,
        mut active_scenes: ResMut<'_, ActiveScenes>,
        mut commands: Commands<'_, '_>,
        entity_with_children_query: Query<'_, '_, &Children>,
    ) {
        for asset_loaded_event in asset_loaded_events.iter() {
            match asset_loaded_event {
                AssetRegistryEvent::AssetLoaded(resource_id)
                    if resource_id.kind == sample_data::runtime::Entity::TYPE =>
                {
                    active_scenes
                        .iter_mut()
                        .filter_map(|(_scene_top_resource, scene)| {
                            if *resource_id == scene.root_resource
                                || scene.asset_to_entity_map.get(*resource_id).is_some()
                            {
                                Some(scene)
                            } else {
                                None
                            }
                        })
                        .for_each(|scene| {
                            scene.spawn_entity_hierarchy(
                                *resource_id,
                                &asset_registry,
                                &mut commands,
                                &entity_with_children_query,
                            );
                        });
                }
                _ => (),
            }
        }
    }
}
