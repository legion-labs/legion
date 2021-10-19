use std::collections::HashMap;

use tonic::{Request, Response, Status};

use legion_editor_proto::{
    editor_server::{Editor, EditorServer},
    GetResourcePropertiesRequest, GetResourcePropertiesResponse, ResourceDescription,
    ResourceProperty, SearchResourcesRequest, SearchResourcesResponse,
    UpdateResourcePropertiesRequest, UpdateResourcePropertiesResponse,
};

pub(crate) struct GRPCServer {
    resources: HashMap<String, GetResourcePropertiesResponse>,
}

impl GRPCServer {
    /// Instanciate a new `GRPCServer` using the specified `webrtc::WebRTCServer`.
    pub(crate) fn new() -> Self {
        let mut resources = HashMap::new();

        resources.insert(
            "triangle".to_string(),
            GetResourcePropertiesResponse {
                description: Some(ResourceDescription {
                    id: "triangle".to_string(),
                    path: "/test/triangle".to_string(),
                    version: 1,
                }),
                properties: vec![
                    ResourceProperty {
                        name: "color".to_string(),
                        ptype: "color".to_string(),
                        group: "material".to_string(),
                        default_value: "\"#FF0000FF\"".to_string().into_bytes(),
                        value: "\"#FF0000FF\"".to_string().into_bytes(),
                    },
                    ResourceProperty {
                        name: "speed".to_string(),
                        ptype: "speed".to_string(),
                        group: "movement".to_string(),
                        default_value: "1.0".to_string().into_bytes(),
                        value: "1.0".to_string().into_bytes(),
                    },
                ],
            },
        );
        resources.insert(
            "polygon".to_string(),
            GetResourcePropertiesResponse {
                description: Some(ResourceDescription {
                    id: "polygon".to_string(),
                    path: "/test/polygon".to_string(),
                    version: 2,
                }),
                properties: vec![
                    ResourceProperty {
                        name: "color".to_string(),
                        ptype: "color".to_string(),
                        group: "material".to_string(),
                        default_value: "\"#FF0000FF\"".to_string().into_bytes(),
                        value: "\"#FF0000FF\"".to_string().into_bytes(),
                    },
                    ResourceProperty {
                        name: "vertex".to_string(),
                        ptype: "u32".to_string(),
                        group: "shape".to_string(),
                        default_value: "4".to_string().into_bytes(),
                        value: "4".to_string().into_bytes(),
                    },
                ],
            },
        );

        Self { resources }
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
        Ok(Response::new(SearchResourcesResponse {
            next_search_token: "".to_string(),
            total: 1,
            resource_descriptions: self
                .resources
                .clone()
                .into_iter()
                .map(|(_, x)| x.description.unwrap())
                .collect(),
        }))
    }

    async fn get_resource_properties(
        &self,
        request: Request<GetResourcePropertiesRequest>,
    ) -> Result<Response<GetResourcePropertiesResponse>, Status> {
        let id = request.into_inner().id;

        match self.resources.get(&id) {
            Some(resp) => Ok(Response::new(resp.clone())),
            None => Err(Status::new(
                tonic::Code::NotFound,
                format!("no such resource `{}`", id),
            )),
        }
    }

    async fn update_resource_properties(
        &self,
        _request: Request<UpdateResourcePropertiesRequest>,
    ) -> Result<Response<UpdateResourcePropertiesResponse>, Status> {
        Ok(Response::new(UpdateResourcePropertiesResponse {}))
    }
}
