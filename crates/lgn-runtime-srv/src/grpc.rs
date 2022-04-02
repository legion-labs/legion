use lgn_async::receiver::SharedUnboundedReceiver;
use lgn_content_store2::ChunkIdentifier;
use lgn_runtime_proto::runtime::{
    runtime_server::{Runtime, RuntimeServer},
    LoadManifestRequest, LoadManifestResponse, LoadRootAssetRequest, LoadRootAssetResponse,
    PauseRequest, PauseResponse,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub(crate) enum ManifestEvent {
    LoadManifest(ChunkIdentifier),
}

pub(crate) type ManifestEventsReceiver = SharedUnboundedReceiver<ManifestEvent>;

pub(crate) struct GRPCServer {
    manifest_events_receiver: ManifestEventsReceiver,
}

impl GRPCServer {
    pub(crate) fn new(manifest_events_receiver: ManifestEventsReceiver) -> Self {
        Self {
            manifest_events_receiver,
        }
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
        todo!()
    }

    async fn load_root_asset(
        &self,
        request: Request<LoadRootAssetRequest>,
    ) -> Result<Response<LoadRootAssetResponse>, Status> {
        todo!()
    }

    async fn pause(
        &self,
        request: Request<PauseRequest>,
    ) -> Result<Response<PauseResponse>, Status> {
        todo!()
    }
}
