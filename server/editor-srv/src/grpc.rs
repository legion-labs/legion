use log::info;
use tonic::{Request, Response, Status};

use legion_editor_proto::{
    editor_server::{Editor, EditorServer},
    GetResourcePropertiesRequest, GetResourcePropertiesResponse, ResourceDescription,
    ResourceProperty, ResourcePropertyUpdate, SearchResourcesRequest, SearchResourcesResponse,
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

        let handle = resource_handles
            .get(resource_id)
            .ok_or_else(|| Status::internal(format!("Invalid ResourceID: {}", resource_id)))?;

        let mut response = GetResourcePropertiesResponse {
            description: Some(ResourceDescription {
                id: resource_id.to_string(),
                path: project
                    .resource_name(resource_id)
                    .unwrap_or_else(|_err| "".into())
                    .to_string(),
                version: 1,
            }),
            properties: Vec::new(),
        };

        // Refresh for Reflection interface. Might not be present for type with no properties
        if let Some(reflection) = registry.get_resource_reflection(resource_id.ty(), handle) {
            let descriptors = reflection.get_property_descriptors().ok_or_else(|| {
                Status::internal(format!(
                    "Invalid Property Descriptor for ResourceId: {}",
                    resource_id
                ))
            })?;

            let properties: Result<Vec<ResourceProperty>, &'static str> = descriptors
                .iter()
                .map(
                    |(_key, descriptor)| -> Result<ResourceProperty, &'static str> {
                        let value = reflection.read_property(descriptor.name)?;

                        let default_value = reflection.read_property_default(descriptor.name)?;

                        return Ok(ResourceProperty {
                            name: descriptor.name.into(),
                            ptype: descriptor.type_name.to_lowercase(),
                            group: descriptor.group.to_string(),
                            default_value: default_value.as_bytes().to_vec(),
                            value: value.as_bytes().to_vec(),
                        });
                    },
                )
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
        let mut project = self.project.lock().unwrap();
        let mut registry = self.registry.lock().unwrap();
        let resource_handles = self.resource_handles.lock().unwrap();

        let request = request.into_inner();

        info!("updating resource properties for entity {}", request.id);

        let resource_id: ResourceId =
            ResourceId::from_str(request.id.as_str()).map_err(|_err| {
                Status::internal(format!("Invalid ResourceID format: {}", request.id))
            })?;

        if let Some(handle) = resource_handles.get(resource_id) {
            if let Some(reflection) = registry.get_resource_reflection_mut(resource_id.ty(), handle)
            {
                let results: Result<Vec<ResourcePropertyUpdate>, &'static str> = request
                    .property_updates
                    .iter()
                    .map(|update| -> Result<ResourcePropertyUpdate, &'static str> {
                        let value = std::str::from_utf8(update.value.as_slice())
                            .map_err(|_err| "invalid value")?;

                        if reflection
                            .write_property(update.name.as_str(), value)
                            .is_ok()
                        {
                            // Read back
                            let value = reflection.read_property(update.name.as_str())?;

                            return Ok(ResourcePropertyUpdate {
                                name: update.name.clone(),
                                value: value.as_bytes().to_vec(),
                            });
                        }
                        Err("property set failed")
                    })
                    .collect();

                if let Ok(properties) = results {
                    project
                        .save_resource(resource_id, handle, &mut registry)
                        .map_err(|err| {
                            Status::internal(format!(
                                "Failed to save ResourceId {}: {}",
                                resource_id, err
                            ))
                        })?;
                    return Ok(Response::new(UpdateResourcePropertiesResponse {
                        version: request.version + 1,
                        updated_properties: properties,
                    }));
                }
            }
        }

        Err(Status::internal(format!(
            "Invalid ResourceID: {}",
            resource_id
        )))
    }
}
