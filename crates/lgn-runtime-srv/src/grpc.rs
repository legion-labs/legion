use crossbeam_channel::Sender;
use lgn_content_store::Identifier;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_runtime_proto::runtime::{
    runtime_server::{Runtime, RuntimeServer},
    LoadManifestRequest, LoadManifestResponse, LoadRootAssetRequest, LoadRootAssetResponse,
    PauseRequest, PauseResponse,
};
use lgn_tracing::warn;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub(crate) enum RuntimeServerCommand {
    LoadManifest(Identifier),
    LoadRootAsset(ResourceTypeAndId),
    Pause,
}

pub(crate) struct GRPCServer {
    command_sender: crossbeam_channel::Sender<RuntimeServerCommand>,
}

impl GRPCServer {
    pub(crate) fn new(command_sender: Sender<RuntimeServerCommand>) -> Self {
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

        let manifest_id = request.manifest_id.parse::<Identifier>().map_err(|error| {
            Status::internal(format!(
                "Invalid manifest id format \"{}\": {}",
                request.manifest_id, error
            ))
        })?;

        if let Err(error) = self
            .command_sender
            .send(RuntimeServerCommand::LoadManifest(manifest_id))
        {
            warn!(
                "internal error sending runtime server command on channel: {}",
                error
            );
        }

        Ok(Response::new(LoadManifestResponse {}))
    }

    async fn load_root_asset(
        &self,
        request: Request<LoadRootAssetRequest>,
    ) -> Result<Response<LoadRootAssetResponse>, Status> {
        let request = request.into_inner();

        let root_asset_id =
            request
                .root_asset_id
                .parse::<ResourceTypeAndId>()
                .map_err(|error| {
                    Status::internal(format!(
                        "Invalid asset id format \"{}\": {}",
                        request.root_asset_id, error
                    ))
                })?;

        if let Err(error) = self
            .command_sender
            .send(RuntimeServerCommand::LoadRootAsset(root_asset_id))
        {
            warn!(
                "internal error sending runtime server command on channel: {}",
                error
            );
        }

        Ok(Response::new(LoadRootAssetResponse {}))
    }

    async fn pause(
        &self,
        request: Request<PauseRequest>,
    ) -> Result<Response<PauseResponse>, Status> {
        let _request = request.into_inner();

        if let Err(error) = self.command_sender.send(RuntimeServerCommand::Pause {}) {
            warn!(
                "internal error sending runtime server command on channel: {}",
                error
            );
        }

        Ok(Response::new(PauseResponse {}))
    }
}
