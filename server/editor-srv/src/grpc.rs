use std::sync::Arc;

use legion_data_runtime::{from_str, to_string, ResourceId, ResourceType};
use legion_editor_proto::{
    editor_server::{Editor, EditorServer},
    GetResourcePropertiesRequest, GetResourcePropertiesResponse, RedoTransactionRequest,
    RedoTransactionResponse, ResourceDescription, ResourceProperty, ResourcePropertyUpdate,
    SearchResourcesRequest, SearchResourcesResponse, UndoTransactionRequest,
    UndoTransactionResponse, UpdateResourcePropertiesRequest, UpdateResourcePropertiesResponse,
};
use log::info;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use legion_data_transaction::{DataManager, LockContext, Transaction};

pub(crate) struct GRPCServer {
    data_manager: Arc<Mutex<DataManager>>,
}

impl GRPCServer {
    /// Instanciate a new `GRPCServer` using the specified `webrtc::WebRTCServer`.
    pub(crate) fn new(data_manager: Arc<Mutex<DataManager>>) -> Self {
        Self { data_manager }
    }

    pub fn service(self) -> EditorServer<Self> {
        EditorServer::new(self)
    }
}

#[tonic::async_trait]
impl Editor for GRPCServer {
    async fn search_resources(
        &self,
        _request: Request<SearchResourcesRequest>,
    ) -> Result<Response<SearchResourcesResponse>, Status> {
        let data_manager = self.data_manager.lock().await;
        let ctx = LockContext::new(&data_manager).await;
        let descriptors: Vec<ResourceDescription> = ctx
            .project
            .resource_list()
            .map(|resource_id| {
                let name = ctx
                    .project
                    .resource_name(resource_id)
                    .unwrap_or_else(|_err| "".into());

                ResourceDescription {
                    id: format!("{}", to_string(*resource_id)),
                    path: name.to_string(),
                    version: 1,
                }
            })
            .collect();

        let response = SearchResourcesResponse {
            next_search_token: "".to_string(),
            total: descriptors.len() as u64,
            resource_descriptions: descriptors,
        };

        Ok(Response::new(response))
    }

    async fn undo_transaction(
        &self,
        _request: Request<UndoTransactionRequest>,
    ) -> Result<Response<UndoTransactionResponse>, Status> {
        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .undo_transaction()
            .await
            .map_err(|err| Status::internal(format!("Undo transaction failed: {}", err)))?;

        Ok(Response::new(UndoTransactionResponse { id: 0 }))
    }

    async fn redo_transaction(
        &self,
        _request: Request<RedoTransactionRequest>,
    ) -> Result<Response<RedoTransactionResponse>, Status> {
        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .redo_transaction()
            .await
            .map_err(|err| Status::internal(format!("Redo transaction failed: {}", err)))?;

        Ok(Response::new(RedoTransactionResponse { id: 0 }))
    }

    async fn get_resource_properties(
        &self,
        request: Request<GetResourcePropertiesRequest>,
    ) -> Result<Response<GetResourcePropertiesResponse>, Status> {
        let resource_id: (ResourceType, ResourceId) = from_str(request.get_ref().id.as_str())
            .map_err(|_err| {
                Status::internal(format!(
                    "Invalid ResourceID format: {}",
                    request.get_ref().id
                ))
            })?;

        let data_manager = self.data_manager.lock().await;
        let ctx = LockContext::new(&data_manager).await;
        let handle = ctx
            .loaded_resource_handles
            .get(resource_id)
            .ok_or_else(|| Status::internal(format!("Invalid ResourceID: {:?}", resource_id)))?;

        let mut response = GetResourcePropertiesResponse {
            description: Some(ResourceDescription {
                id: format!("{}", to_string(resource_id)),
                path: ctx
                    .project
                    .resource_name(resource_id)
                    .unwrap_or_else(|_err| "".into())
                    .to_string(),
                version: 1,
            }),
            properties: Vec::new(),
        };

        // Refresh for Reflection interface. Might not be present for type with no properties
        if let Some(reflection) = ctx
            .resource_registry
            .get_resource_reflection(resource_id.0, handle)
        {
            let descriptors = reflection.get_property_descriptors().ok_or_else(|| {
                Status::internal(format!(
                    "Invalid Property Descriptor for ResourceId: {:?}",
                    resource_id
                ))
            })?;

            let properties: anyhow::Result<Vec<ResourceProperty>> = descriptors
                .iter()
                .map(|(_key, descriptor)| -> anyhow::Result<ResourceProperty> {
                    let value = reflection.read_property(descriptor.name)?;

                    let default_value = reflection.read_property_default(descriptor.name)?;

                    return Ok(ResourceProperty {
                        name: descriptor.name.into(),
                        ptype: descriptor.type_name.to_lowercase(),
                        group: descriptor.group.to_string(),
                        default_value: default_value.as_bytes().to_vec(),
                        value: value.as_bytes().to_vec(),
                    });
                })
                .collect();

            if let Ok(properties) = properties {
                response.properties = properties;
            }
        }

        Ok(Response::new(response))
    }

    async fn update_resource_properties(
        &self,
        request: Request<UpdateResourcePropertiesRequest>,
    ) -> Result<Response<UpdateResourcePropertiesResponse>, Status> {
        let request = request.into_inner();

        info!("updating resource properties for entity {}", request.id);

        let resource_id: (ResourceType, ResourceId) =
            from_str(request.id.as_str()).map_err(|_err| {
                Status::internal(format!("Invalid ResourceID format: {}", request.id))
            })?;

        let mut data_manager = self.data_manager.lock().await;
        {
            let mut transaction = Transaction::new();
            request
                .property_updates
                .iter()
                .try_for_each(|update| -> anyhow::Result<()> {
                    let value = std::str::from_utf8(update.value.as_slice())?;
                    transaction.update_property(resource_id, update.name.as_str(), value)
                })
                .map_err(|err| Status::internal(format!("transaction error {}", err)))?;
            data_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Status::internal(format!("transaction error {}", err)))?;
        }

        let ctx = LockContext::new(&data_manager).await;
        let handle = ctx
            .loaded_resource_handles
            .get(resource_id)
            .ok_or_else(|| {
                Status::internal(format!("Invalid ResourceID: {}", to_string(resource_id)))
            })?;

        let reflection = ctx
            .resource_registry
            .get_resource_reflection(resource_id.0, handle)
            .ok_or_else(|| {
                Status::internal(format!("Invalid ResourceID: {}", to_string(resource_id)))
            })?;

        let results: anyhow::Result<Vec<ResourcePropertyUpdate>> = request
            .property_updates
            .iter()
            .map(|update| -> anyhow::Result<ResourcePropertyUpdate> {
                Ok(ResourcePropertyUpdate {
                    name: update.name.clone(),
                    value: reflection
                        .read_property(update.name.as_str())?
                        .as_bytes()
                        .to_vec(),
                })
            })
            .collect();

        if let Ok(properties) = results {
            return Ok(Response::new(UpdateResourcePropertiesResponse {
                version: request.version + 1,
                updated_properties: properties,
            }));
        }

        Err(Status::internal(format!(
            "Invalid ResourceID: {:?}",
            resource_id
        )))
    }
}
