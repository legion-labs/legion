#![allow(unused_imports)]
use std::{collections::HashSet, sync::Arc};

use lgn_app::{prelude::*, Events};
use lgn_asset_registry::AssetToEntityMap;
use lgn_async::TokioAsyncRuntime;
use lgn_core::Name;
use lgn_data_offline::ResourcePathId;
use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};
use lgn_data_transaction::{
    LockContext, SelectionManager, SelectionOperation, Transaction, TransactionManager,
    UpdatePropertyOperation,
};
use lgn_ecs::prelude::*;
use lgn_input::{
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButtonInput, MouseMotion},
    Input,
};
use lgn_renderer::picking::PickingEvent;
use lgn_renderer::picking::{ManipulatorManager, PickingManager};
use lgn_tracing::{error, info, warn};
use lgn_transform::components::Transform;
use tokio::sync::Mutex;

use crate::source_control_plugin::{RawFilesStreamerConfig, SharedRawFilesStreamer};

#[derive(Default)]
pub struct EditorPlugin;

// This is our event that we will send and receive in systems

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(SelectionManager::create())
        .insert_resource(SharedRawFilesStreamer::default())
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
        transaction_manager: Res<'_, Arc<Mutex<TransactionManager>>>,
        mut grpc_settings: ResMut<'_, lgn_grpc::GRPCPluginSettings>,
    ) {
        let grpc_server = super::grpc::GRPCServer::new(transaction_manager.clone());
        grpc_settings.register_service(grpc_server.service());
    }

    #[allow(clippy::needless_pass_by_value)]
    fn update_selection(
        selection_manager: Res<'_, Arc<SelectionManager>>,
        picking_manager: Res<'_, PickingManager>,
        asset_to_entity_map: Res<'_, AssetToEntityMap>,
    ) {
        if let Some(selection) = selection_manager.update() {
            // Convert the SelectionManager offlineId to RuntimeId
            let entities_selection = selection
                .iter()
                .filter_map(|offline_id| {
                    let runtime_id = ResourcePathId::from(*offline_id)
                        .push(sample_data::runtime::Entity::TYPE)
                        .resource_id();
                    asset_to_entity_map.get(runtime_id)
                })
                .collect();
            picking_manager.set_active_selection(entities_selection);
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn process_input(
        tokio_runtime: ResMut<'_, TokioAsyncRuntime>,
        transaction_manager: Res<'_, Arc<Mutex<TransactionManager>>>,
        asset_to_entity_map: Res<'_, AssetToEntityMap>,
        entities: Query<'_, '_, (Entity, Option<&Name>)>,
        mut event_reader: EventReader<'_, '_, PickingEvent>,
        keys: Res<'_, Input<KeyCode>>,
    ) {
        if keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::Z) {
            let transaction_manager = transaction_manager.clone();
            tokio_runtime.start_detached(async move {
                let mut transaction_manager = transaction_manager.lock().await;
                if let Err(err) = transaction_manager.undo_transaction().await {
                    error!("Undo transaction failed: {}", err);
                }
            });
        } else if keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::Y) {
            let transaction_manager = transaction_manager.clone();
            tokio_runtime.start_detached(async move {
                let mut transaction_manager = transaction_manager.lock().await;
                if let Err(err) = transaction_manager.redo_transaction().await {
                    error!("Redo transaction failed: {}", err);
                }
            });
        }

        for event in event_reader.iter() {
            match event {
                PickingEvent::EntityPicked(id) => {
                    if let Some(runtime_id) = asset_to_entity_map.get_resource_id(*id) {
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
                        if let Some(runtime_id) = asset_to_entity_map.get_resource_id(entity) {
                            let position_value =
                                serde_json::json!(transform.translation).to_string();
                            let rotation_value = serde_json::json!(transform.rotation).to_string();
                            let scale_value = serde_json::json!(transform.scale).to_string();

                            let transaction_manager = transaction_manager.clone();

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
                                    if let Err(err) =
                                        transaction_manager.commit_transaction(transaction).await
                                    {
                                        error!("ApplyTransform transaction failed: {}", err);
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
