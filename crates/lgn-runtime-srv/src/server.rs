use async_trait::async_trait;
use crossbeam_channel::Sender;
use lgn_content_store::indexing::TreeIdentifier;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_online::server::{Error, Result};
use lgn_tracing::warn;

use crate::api::runtime::{
    server::{
        LoadManifestRequest, LoadManifestResponse, LoadRootAssetRequest, LoadRootAssetResponse,
        PauseRequest, PauseResponse,
    },
    Api,
};

#[derive(Debug, Clone)]
pub(crate) enum RuntimeServerCommand {
    LoadManifest(TreeIdentifier),
    LoadRootAsset(ResourceTypeAndId),
    Pause,
}

/// The `api` implementation for the runtime server.
pub(crate) struct Server {
    command_sender: Sender<RuntimeServerCommand>,
}

impl Server {
    pub(crate) fn new(command_sender: Sender<RuntimeServerCommand>) -> Self {
        Self { command_sender }
    }
}

#[async_trait]
impl Api for Server {
    async fn load_manifest(&self, request: LoadManifestRequest) -> Result<LoadManifestResponse> {
        let input = std::str::from_utf8(&request.body.0)
            .map_err(|err| Error::internal(format!("Invalid input: {}", err)))?;

        let manifest_id = input
            .to_owned()
            .parse::<TreeIdentifier>()
            .map_err(|error| {
                Error::internal(format!(
                    "Invalid manifest id format \"{}\": {}",
                    String::from_utf8(request.body.0).unwrap(),
                    error
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

        Ok(LoadManifestResponse::Status204 {})
    }

    async fn load_root_asset(
        &self,
        request: LoadRootAssetRequest,
    ) -> Result<LoadRootAssetResponse> {
        let input = std::str::from_utf8(&request.body.0)
            .map_err(|err| Error::internal(format!("Invalid input: {}", err)))?;

        let root_asset_id = input
            .to_owned()
            .parse::<ResourceTypeAndId>()
            .map_err(|error| {
                Error::internal(format!(
                    "Invalid asset id format \"{}\": {}",
                    String::from_utf8(request.body.0).unwrap(),
                    error
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

        Ok(LoadRootAssetResponse::Status204 {})
    }

    async fn pause(&self, _request: PauseRequest) -> Result<PauseResponse> {
        if let Err(error) = self.command_sender.send(RuntimeServerCommand::Pause {}) {
            warn!(
                "internal error sending runtime server command on channel: {}",
                error
            );
        }

        Ok(PauseResponse::Status204 {})
    }
}
