//! Scene/Entity manager

// crate-specific lint exceptions:
//#![allow()]

use std::sync::Arc;

use lgn_app::{App, Plugin};
use lgn_async::TokioAsyncRuntime;
use lgn_data_runtime::prelude::*;

use lgn_ecs::prelude::*;

mod scene_instance;
mod scene_manager;
pub use scene_manager::*;

use sample_data::runtime::Entity;

/// Message Scene Management
pub enum SceneMessage {
    OpenScene(ResourceTypeAndId),
    CloseScene(ResourceTypeAndId),
}

#[derive(Component)]
pub struct ResourceMetaInfo {
    pub id: ResourceTypeAndId,
}

pub struct ScenePlugin {
    startup_scene: Option<ResourceTypeAndId>,
}

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::handle_scene_messages)
            .insert_resource(SceneManager::new())
            .add_event::<SceneMessage>();

        if let Some(scene_id) = self.startup_scene {
            app.world
                .get_resource_mut::<lgn_ecs::event::Events<SceneMessage>>()
                .unwrap()
                .send(SceneMessage::OpenScene(scene_id));
        }
    }
}

impl ScenePlugin {
    /// Init a new `ScenePlugin`
    pub fn new(startup_scene: Option<ResourceTypeAndId>) -> Self {
        Self { startup_scene }
    }

    #[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
    fn handle_scene_messages(
        asset_registry: Res<'_, Arc<AssetRegistry>>,
        mut scene_events: EventReader<'_, '_, SceneMessage>,
        scene_manager: Res<'_, Arc<SceneManager>>,
        tokio_runtime: ResMut<'_, TokioAsyncRuntime>,
        mut commands: Commands<'_, '_>,
    ) {
        for event in scene_events.iter() {
            match event {
                SceneMessage::OpenScene(resource_id) => {
                    let asset_registry = asset_registry.clone();
                    let resource_id = *resource_id;
                    let scene_manager = scene_manager.clone();
                    tokio_runtime.start_detached(async move {
                        match asset_registry.load_async::<Entity>(resource_id).await {
                            Ok(handle) => {
                                scene_manager.add_pending(handle);
                                println!("ok");
                            }
                            Err(err) => lgn_tracing::error!(
                                "Error Loading {}: {}",
                                resource_id,
                                err.to_string()
                            ),
                        }
                    });
                }
                SceneMessage::CloseScene(resource_id) => {
                    scene_manager.close_scene(resource_id, &mut commands);
                }
            }
        }
        scene_manager.update(&asset_registry, &mut commands);
    }
}
