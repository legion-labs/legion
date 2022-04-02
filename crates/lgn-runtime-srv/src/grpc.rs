use std::str::FromStr;

use lgn_content_store2::ChunkIdentifier;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_runtime_proto::runtime::{
    runtime_server::{Runtime, RuntimeServer},
    LoadManifestRequest, LoadManifestResponse, LoadRootAssetRequest, LoadRootAssetResponse,
    PauseRequest, PauseResponse,
};
use lgn_tracing::warn;
use tokio::sync::broadcast;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub(crate) enum RuntimeServerCommand {
    LoadManifest(ChunkIdentifier),
    LoadRootAsset(ResourceTypeAndId),
    Pause,
}

pub(crate) struct GRPCServer {
    command_sender: broadcast::Sender<RuntimeServerCommand>,
}

impl GRPCServer {
    pub(crate) fn new(command_sender: broadcast::Sender<RuntimeServerCommand>) -> Self {
        Self { command_sender }
    }

    pub(crate) fn service(self) -> RuntimeServer<Self> {
        RuntimeServer::new(self)
    }
}

#[tonic::async_trait]
impl Runtime for GRPCServer {
    async fn load_manifest(
        &self,
        request: Request<LoadManifestRequest>,
    ) -> Result<Response<LoadManifestResponse>, Status> {
        let request = request.into_inner();
        if let Ok(manifest_id) = ChunkIdentifier::from_str(&request.manifest_id) {
            if let Err(_error) = self
                .command_sender
                .send(RuntimeServerCommand::LoadManifest(manifest_id))
            {
                warn!("internal error sending runtime server command on channel");
            }
        }
        Ok(Response::new(LoadManifestResponse {}))
    }

    async fn load_root_asset(
        &self,
        request: Request<LoadRootAssetRequest>,
    ) -> Result<Response<LoadRootAssetResponse>, Status> {
        let request = request.into_inner();
        if let Ok(root_asset_id) = ResourceTypeAndId::from_str(&request.root_asset_id) {
            if let Err(_error) = self
                .command_sender
                .send(RuntimeServerCommand::LoadRootAsset(root_asset_id))
            {
                warn!("internal error sending runtime server command on channel");
            }
        }
        Ok(Response::new(LoadRootAssetResponse {}))
    }

    async fn pause(
        &self,
        request: Request<PauseRequest>,
    ) -> Result<Response<PauseResponse>, Status> {
        let _request = request.into_inner();
        if let Err(_error) = self.command_sender.send(RuntimeServerCommand::Pause {}) {
            warn!("internal error sending runtime server command on channel");
        }
        Ok(Response::new(PauseResponse {}))
    }
}
