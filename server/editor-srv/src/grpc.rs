use tonic::{Request, Response, Status};

use legion_editor_proto::{
    editor_server::{Editor, EditorServer},
    GetResourcePropertiesRequest, GetResourcePropertiesResponse, ResourceDescription,
    ResourceProperty, SearchResourcesRequest, SearchResourcesResponse,
    UpdateResourcePropertiesRequest, UpdateResourcePropertiesResponse,
};

pub(crate) struct GRPCServer {}

impl GRPCServer {
    /// Instanciate a new `GRPCServer` using the specified `webrtc::WebRTCServer`.
    pub(crate) fn new() -> Self {
        Self {}
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
            resource_descriptions: vec![ResourceDescription {
                id: "myresource".to_string(),
                path: "/path/to/my/resource".to_string(),
                version: 1,
            }],
        }))
    }

    async fn get_resource_properties(
        &self,
        _request: Request<GetResourcePropertiesRequest>,
    ) -> Result<Response<GetResourcePropertiesResponse>, Status> {
        Ok(Response::new(GetResourcePropertiesResponse {
            description: Some(ResourceDescription {
                id: "myresource".to_string(),
                path: "/path/to/my/resource".to_string(),
                version: 1,
            }),
            properties: vec![ResourceProperty {
                name: "color".to_string(),
                ptype: "color".to_string(),
                group: "material".to_string(),
                default_value: vec![],
                value: vec![],
            }],
        }))
    }

    async fn update_resource_properties(
        &self,
        _request: Request<UpdateResourcePropertiesRequest>,
    ) -> Result<Response<UpdateResourcePropertiesResponse>, Status> {
        Ok(Response::new(UpdateResourcePropertiesResponse {}))
    }
}
