use tonic::{Request, Response, Status};

use legion_editor_proto::{
    editor_server::{Editor, EditorServer},
    GetResourcePropertiesRequest, GetResourcePropertiesResponse, ResourceDescription,
    ResourceProperty, SearchResourcesRequest, SearchResourcesResponse,
    UpdateResourcePropertiesRequest, UpdateResourcePropertiesResponse,
};

use legion_data_offline::resource::{Project, ResourceHandles, ResourceRegistry};
use legion_data_runtime::ResourceId;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

pub(crate) struct GRPCServer {
    project: Arc<Mutex<Project>>,
    registry: Arc<Mutex<ResourceRegistry>>,
    resource_handles: Arc<Mutex<ResourceHandles>>,
}

impl GRPCServer {
    /// Instanciate a new `GRPCServer` using the specified `webrtc::WebRTCServer`.
    pub(crate) fn new(
        project: Arc<Mutex<Project>>,
        registry: Arc<Mutex<ResourceRegistry>>,
        resource_handles: Arc<Mutex<ResourceHandles>>,
    ) -> Self {
        Self {
            project,
            registry,
            resource_handles,
        }
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
        let project = self.project.lock().unwrap();

        let descriptors: Vec<ResourceDescription> = project
            .resource_list()
            .iter()
            .map(|resource_id| {
                let name = project
                    .resource_name(*resource_id)
                    .unwrap_or_else(|_err| "".into());

                ResourceDescription {
                    id: resource_id.to_string(),
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

    async fn get_resource_properties(
        &self,
        request: Request<GetResourcePropertiesRequest>,
    ) -> Result<Response<GetResourcePropertiesResponse>, Status> {
        let project = self.project.lock().unwrap();
        let registry = self.registry.lock().unwrap();
        let resource_handles = self.resource_handles.lock().unwrap();

        let resource_id: ResourceId =
            ResourceId::from_str(request.get_ref().id.as_str()).map_err(|_err| {
                Status::internal(format!(
                    "Invalid ResourceID format: {}",
                    request.get_ref().id
                ))
            })?;

        if let Some(handle) = resource_handles.get(resource_id) {
            let name = project
                .resource_name(resource_id)
                .unwrap_or_else(|_err| "".into());

            let header = ResourceDescription {
                id: resource_id.to_string(),
                path: name.to_string(),
                version: 1,
            };

            let response = GetResourcePropertiesResponse {
                description: Some(header),
                properties: registry
                    .get_resource_properties(resource_id.ty(), handle)
                    .map_err(Status::internal)?
                    .iter()
                    .map(|property| ResourceProperty {
                        name: property.name.into(),
                        ptype: property.type_name.into(),
                        group: property.group.to_string(),
                        default_value: property.default_value.clone(),
                        value: property.value.clone(),
                    })
                    .collect(),
            };
            return Ok(Response::new(response));
        }
        Err(Status::internal(format!(
            "Invalid ResourceID: {}",
            resource_id
        )))
    }

    async fn update_resource_properties(
        &self,
        _request: Request<UpdateResourcePropertiesRequest>,
    ) -> Result<Response<UpdateResourcePropertiesResponse>, Status> {
        //let project = *self.project.lock().unwrap();
        //let registry = *self.registry.lock().unwrap();
        //let resource_handles = *self.resource_handles.lock().unwrap();

        Ok(Response::new(UpdateResourcePropertiesResponse {}))
    }
}
