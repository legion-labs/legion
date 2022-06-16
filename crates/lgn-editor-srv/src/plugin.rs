#![allow(unused_imports)]
use std::{collections::HashSet, sync::Arc};

use editor_srv::editor::server::register_routes;
use lgn_app::{prelude::*, Events};
use lgn_asset_registry::AssetToEntityMap;
use lgn_async::TokioAsyncRuntime;
use lgn_core::Name;
use lgn_data_runtime::{
    Resource, ResourceDescriptor, ResourceId, ResourcePathId, ResourceTypeAndId,
};
use lgn_data_transaction::{
    LockContext, SelectionManager, SelectionOperation, Transaction, TransactionManager,
    UpdatePropertyOperation,
};
use lgn_ecs::prelude::*;
use lgn_graphics_renderer::picking::PickingEvent;
use lgn_graphics_renderer::picking::{ManipulatorManager, PickingManager};
use lgn_grpc::SharedRouter;
use lgn_input::{
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButtonInput, MouseMotion},
    Input,
};
use lgn_scene_plugin::ActiveScenes;
use lgn_tracing::{error, info, warn};
use lgn_transform::components::Transform;
use tokio::sync::{broadcast, Mutex};

use crate::editor::{EditorEvent, EditorEventsReceiver, Server};
use crate::source_control_plugin::{RawFilesStreamerConfig, SharedRawFilesStreamer};

#[derive(Default)]
pub struct EditorPlugin;

// This is our event that we will send and receive in systems

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        let (editor_events_sender, editor_events_receiver) = broadcast::channel(1_000);
        let editor_events_receiver: EditorEventsReceiver = editor_events_receiver.into();

        app
        .insert_resource(SelectionManager::create())
        .init_resource::<SharedRawFilesStreamer>()
        .insert_resource(editor_events_sender)
        .insert_resource(editor_events_receiver)
        .add_system_to_stage(CoreStage::PostUpdate, Self::process_input)
        .add_system_to_stage(CoreStage::PostUpdate, Self::update_selection)
        .add_startup_system_to_stage(
            StartupStage::PostStartup,
            Self::setup
                .exclusive_system()
                .after(lgn_resource_registry::ResourceRegistryPluginScheduling::ResourceRegistryCreated)
                .before(lgn_grpc::GRPCPluginScheduling::StartRpcServer),
        );
    }
}

impl EditorPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn setup(
        mut router: ResMut<'_, SharedRouter>,
        transaction_manager: Res<'_, Arc<Mutex<TransactionManager>>>,
        editor_events_receiver: Res<'_, EditorEventsReceiver>,
    ) {
        let server = Arc::new(Server::new(
            transaction_manager.clone(),
            editor_events_receiver.clone(),
        ));

        router.register_routes(register_routes, server);
    }

    #[allow(clippy::needless_pass_by_value)]
    fn update_selection(
        selection_manager: Res<'_, Arc<SelectionManager>>,
        picking_manager: Res<'_, PickingManager>,
        active_scenes: Res<'_, ActiveScenes>,
        event_sender: Res<'_, broadcast::Sender<EditorEvent>>,
    ) {
        if let Some(selection) = selection_manager.update() {
            // Convert the SelectionManager offlineId to RuntimeId
            let entities_selection = selection
                .iter()
                .map(|offline_id| {
                    ResourcePathId::from(*offline_id)
                        .push(sample_data::runtime::Entity::TYPE)
                        .resource_id()
                })
                .flat_map(|runtime_id| {
                    active_scenes
                        .iter()
                        .filter_map(|(_, scene)| scene.asset_to_entity_map.get(runtime_id))
                        .collect::<Vec<_>>()
                })
                .collect();

            if let Err(err) = event_sender.send(EditorEvent::SelectionChanged(selection)) {
                warn!("Failed to send selectionEvent: {}", err);
            }
            picking_manager.set_active_selection(entities_selection);
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn process_input(
        tokio_runtime: ResMut<'_, TokioAsyncRuntime>,
        transaction_manager: Res<'_, Arc<Mutex<TransactionManager>>>,
        active_scenes: Res<'_, ActiveScenes>,
        entities: Query<'_, '_, (Entity, Option<&Name>)>,
        mut event_reader: EventReader<'_, '_, PickingEvent>,
        keys: Res<'_, Input<KeyCode>>,
        event_sender: Res<'_, broadcast::Sender<EditorEvent>>,
    ) {
        if keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::Z) {
            let transaction_manager = transaction_manager.clone();
            let event_sender = event_sender.clone();
            tokio_runtime.start_detached(async move {
                let mut transaction_manager = transaction_manager.lock().await;
                match transaction_manager.undo_transaction().await {
                    Ok(Some(changed)) => {
                        if let Err(err) = event_sender.send(EditorEvent::ResourceChanged(changed)) {
                            warn!("Failed to send EditorEvent: {}", err);
                        }
                    }
                    Err(err) => error!("Undo transaction failed: {}", err),
                    Ok(_) => {}
                }
            });
        } else if keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::Y) {
            let transaction_manager = transaction_manager.clone();
            let event_sender = event_sender.clone();
            tokio_runtime.start_detached(async move {
                let mut transaction_manager = transaction_manager.lock().await;
                match transaction_manager.redo_transaction().await {
                    Ok(Some(changed)) => {
                        if let Err(err) = event_sender.send(EditorEvent::ResourceChanged(changed)) {
                            warn!("Failed to send EditorEvent: {}", err);
                        }
                    }
                    Err(err) => error!("Redo transaction failed: {}", err),
                    Ok(_) => {}
                }
            });
        }

        for event in event_reader.iter() {
            match event {
                PickingEvent::EntityPicked(id) => {
                    if let Some(runtime_id) = active_scenes
                        .iter()
                        .find_map(|(_, scene)| scene.asset_to_entity_map.get_resource_id(*id))
                    {
                        let shift_pressed = false; //TODO: Support adding to selection keys.pressed(KeyCode::LShift);

                        let transaction_manager = transaction_manager.clone();
                        tokio_runtime.start_detached(async move {
                            let mut transaction_manager = transaction_manager.lock().await;
                            let offline_res_id = {
                                let ctx = LockContext::new(&transaction_manager).await;
                                ctx.build.resolve_offline_id(runtime_id).await
                            };
                            if let Some(offline_res_id) = offline_res_id {
                                let transaction =
                                    Transaction::new().add_operation(if shift_pressed {
                                        SelectionOperation::toggle_selection(&[offline_res_id])
                                    } else {
                                        SelectionOperation::set_selection(&[offline_res_id])
                                    });

                                if let Err(err) =
                                    transaction_manager.commit_transaction(transaction).await
                                {
                                    error!("Selection transaction failed: {}", err);
                                }
                            }
                        });
                    }
                }
                PickingEvent::ApplyTransaction(id, transform) => {
                    if let Ok((entity, _name)) = entities.get(*id) {
                        if let Some(runtime_id) = active_scenes.iter().find_map(|(_, scene)| {
                            scene.asset_to_entity_map.get_resource_id(entity)
                        }) {
                            let position_value =
                                serde_json::json!(transform.translation).to_string();
                            let rotation_value = serde_json::json!(transform.rotation).to_string();
                            let scale_value = serde_json::json!(transform.scale).to_string();

                            let transaction_manager = transaction_manager.clone();
                            let event_sender = event_sender.clone();

                            tokio_runtime.start_detached(async move {
                                let mut transaction_manager = transaction_manager.lock().await;
                                let offline_res_id = {
                                    let ctx = LockContext::new(&transaction_manager).await;
                                    ctx.build.resolve_offline_id(runtime_id).await
                                };

                                if let Some(offline_res_id) = offline_res_id {
                                    let transaction = Transaction::new().add_operation(
                                        UpdatePropertyOperation::new(
                                            offline_res_id,
                                            &[
                                                ("components[Transform].position", position_value),
                                                ("components[Transform].rotation", rotation_value),
                                                ("components[Transform].scale", scale_value),
                                            ],
                                        ),
                                    );
                                    match transaction_manager.commit_transaction(transaction).await
                                    {
                                        Ok(Some(changed)) => {
                                            if let Err(err) = event_sender
                                                .send(EditorEvent::ResourceChanged(changed))
                                            {
                                                warn!("Failed to send EditorEvent: {}", err);
                                            }
                                        }
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("ApplyTransform transaction failed: {}", err);
                                        }
                                    }
                                }
                            });
                        }
                    } else {
                        warn!("ApplyTransform failed, entity {:?} not found", id);
                    }
                }
                PickingEvent::ClearSelection => {
                    let transaction_manager = transaction_manager.clone();
                    tokio_runtime.start_detached(async move {
                        let transaction = Transaction::new()
                            .add_operation(SelectionOperation::set_selection(&[]));

                        let mut transaction_manager = transaction_manager.lock().await;
                        if let Err(err) = transaction_manager.commit_transaction(transaction).await
                        {
                            error!("Selec transaction failed: {}", err);
                        }
                    });

                    info!("ClearSelection");
                }
            }
        }
    }
}
