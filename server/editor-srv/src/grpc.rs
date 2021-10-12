use std::net::SocketAddr;

use tonic::{Request, Response, Status};

use legion_editor_proto::{
    editor_server::{Editor, EditorServer},
    GetResourcePropertiesRequest, GetResourcePropertiesResponse, ResourceDescription,
    ResourceProperty, SearchResourcesRequest, SearchResourcesResponse,
    UpdateResourcePropertiesRequest, UpdateResourcePropertiesResponse,
};

use log::{info, warn};

pub(crate) struct GRPCServer {}

impl GRPCServer {
    /// Instanciate a new `GRPCServer` using the specified `webrtc::WebRTCServer`.
    pub(crate) fn new() -> Self {
        Self {}
    }

    /// Start the `gRPC` server on the specified `addr`.
    pub async fn listen_and_serve(self, addr: SocketAddr) -> Result<(), tonic::transport::Error> {
        info!("gRPC server started and listening on {}.", addr);

        match tonic::transport::Server::builder()
            .add_service(EditorServer::new(self))
            .serve(addr)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("gRPC server stopped and no longer listening ({})", e);

                Err(e)
            }
        }
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
