use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use bytes::BytesMut;
use lgn_app::prelude::*;
use lgn_data_offline::resource::ChangeType;
use lgn_data_runtime::{ResourceDescriptor, ResourceTypeAndId};
use lgn_data_transaction::{LockContext, TransactionManager};
use lgn_ecs::{
    prelude::{IntoExclusiveSystem, Res, ResMut},
    schedule::ExclusiveSystemDescriptorCoercion,
};
use lgn_editor_proto::source_control::{
    source_control_server::{SourceControl, SourceControlServer},
    upload_raw_file_response, CancelUploadRawFileRequest, CancelUploadRawFileResponse,
    CommitStagedResourcesRequest, CommitStagedResourcesResponse, GetStagedResourcesRequest,
    GetStagedResourcesResponse, InitUploadRawFileRequest, InitUploadRawFileResponse,
    PullAssetRequest, PullAssetResponse, RevertResourcesRequest, RevertResourcesResponse,
    SyncLatestResponse, SyncLatestResquest, UploadRawFileProgress, UploadRawFileRequest,
    UploadRawFileResponse, UploadStatus,
};
use lgn_graphics_data::offline_gltf::GltfFile;
use lgn_grpc::{GRPCPluginScheduling, GRPCPluginSettings};
use lgn_resource_registry::{ResourceRegistryPluginScheduling, ResourceRegistrySettings};
use lgn_tracing::error;
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{codegen::StdError, Request, Response, Status};
use uuid::Uuid;

#[derive(Error, Debug)]
pub(crate) enum SourceControlError {
    #[error("Couldn't create file at {0}: {1}")]
    FileCreation(PathBuf, StdError),

    #[error("Couldn't write into file: {0}")]
    FileWrite(StdError),

    #[error("Id {0} doesn't exist")]
    UnknownId(FileId),

    #[error("Content is bigger than the status size limit: {content_len} > {max_chunk_size}")]
    ContentSize {
        content_len: usize,
        max_chunk_size: usize,
    },
}

type SourceControlResult<T> = Result<T, SourceControlError>;

#[derive(Hash, PartialEq, Debug, Eq, Clone)]
pub(crate) struct FileId(String);

impl FileId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl From<String> for FileId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<FileId> for String {
    fn from(file_id: FileId) -> Self {
        file_id.0
    }
}

impl Display for FileId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) struct RawFile {
    file: File,
    bytes: BytesMut,
    current_size: usize,
    file_size: usize,
}

/// A `RawFile` is actually a folder under the hood, which name is the file id
/// The uploaded file will be created inside the folder
impl RawFile {
    pub fn new(
        file_id: &FileId,
        path: impl Into<PathBuf>,
        name: impl Into<String>,
        file_size: usize,
        buffer_size: usize,
    ) -> SourceControlResult<Self> {
        let name = name.into();

        let path: PathBuf = path.into();

        let dir_path = path.join(file_id.to_string());

        fs::create_dir_all(&dir_path)
            .map_err(|error| SourceControlError::FileCreation(dir_path.clone(), Box::new(error)))?;

        let file_path = dir_path.join(&name);

        let file = File::create(&file_path)
            .map_err(|error| SourceControlError::FileCreation(file_path, Box::new(error)))?;

        Ok(Self {
            file,
            bytes: BytesMut::with_capacity(buffer_size),
            current_size: 0,
            file_size,
        })
    }

    pub fn flush(&mut self) -> SourceControlResult<()> {
        self.file
            .write(&self.bytes)
            .map_err(|error| SourceControlError::FileWrite(Box::new(error)))?;

        self.clear();

        Ok(())
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
    }

    pub fn extend_from_slice(&mut self, content: &[u8]) -> SourceControlResult<()> {
        // Flush file so it doesn't overflow
        if self.overflows(content.len()) {
            self.flush()?;
        }

        self.bytes.extend_from_slice(content);

        self.current_size += content.len();

        Ok(())
    }

    pub fn overflows(&self, size: usize) -> bool {
        self.bytes.len() + size > self.bytes.capacity()
    }

    pub fn is_complete(&self) -> bool {
        self.current_size == self.file_size
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn completion(&self) -> f32 {
        (self.current_size as f64 / self.file_size as f64 * 100f64) as f32
    }
}

#[derive(Debug)]
pub(crate) enum RawFileStatus {
    InProgress { completion: f32 },
    Done,
    Error { error: SourceControlError },
}

#[derive(Default)]
pub(crate) struct RawFilesHashMap(HashMap<FileId, RawFile>);

impl RawFilesHashMap {
    pub fn contains_key(&self, id: &FileId) -> bool {
        self.0.contains_key(id)
    }

    pub fn insert(&mut self, id: FileId, raw_file: RawFile) -> Option<RawFile> {
        self.0.insert(id, raw_file)
    }

    pub fn remove(&mut self, id: &FileId) -> Option<RawFile> {
        self.0.remove(id)
    }

    pub fn push_in(&mut self, id: &FileId, content: &[u8]) -> SourceControlResult<RawFileStatus> {
        let raw_file = self
            .0
            .get_mut(id)
            .ok_or_else(|| SourceControlError::UnknownId(id.clone()))?;

        raw_file.extend_from_slice(content)?;

        if raw_file.is_complete() {
            raw_file.flush()?;

            self.remove(id);

            return Ok(RawFileStatus::Done);
        };

        Ok(RawFileStatus::InProgress {
            completion: raw_file.completion(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct RawFilesStreamerConfig {
    max_size: u64,
    max_chunk_size: u64,
    max_buffer_size: u64,
}

impl Default for RawFilesStreamerConfig {
    fn default() -> Self {
        Self {
            // 50mb
            max_size: 50_000_000,
            // 10mb
            max_chunk_size: 10_000_000,
            // 10mb
            max_buffer_size: 10_000_000,
        }
    }
}

pub(crate) struct RawFilesStreamer {
    config: RawFilesStreamerConfig,
    storage: RawFilesHashMap,
}

impl RawFilesStreamer {
    pub fn new(config: RawFilesStreamerConfig) -> Self {
        Self {
            config,
            storage: RawFilesHashMap::default(),
        }
    }

    pub fn max_size(&self) -> u64 {
        self.config.max_size
    }

    pub fn max_chunk_size(&self) -> u64 {
        self.config.max_chunk_size
    }

    pub fn insert(
        &mut self,
        path: impl Into<PathBuf>,
        file_name: impl Into<String>,
        size: u64,
    ) -> SourceControlResult<FileId> {
        let file_id = FileId::new();

        self.storage.insert(
            file_id.clone(),
            RawFile::new(
                &file_id,
                path,
                file_name,
                size as usize,
                self.config.max_buffer_size as usize,
            )?,
        );

        // TODO: Auto cancel after Xs

        Ok(file_id)
    }

    pub fn contains_key(&self, id: &FileId) -> bool {
        self.storage.contains_key(id)
    }

    pub async fn push(
        &mut self,
        id: &FileId,
        content: &[u8],
    ) -> SourceControlResult<RawFileStatus> {
        let content_len = content.len();
        let max_chunk_size = self.max_chunk_size() as usize;

        if content_len > max_chunk_size as usize {
            return Err(SourceControlError::ContentSize {
                content_len,
                max_chunk_size,
            });
        }

        self.storage.push_in(id, content)
    }

    pub fn cancel(&mut self, id: &FileId) -> bool {
        self.storage.remove(id).is_some()
    }
}

/// Shared version of [`RawFilesStreamer`], can be safely cloned, sent to threads and mutated
pub(crate) struct SharedRawFilesStreamer(Arc<Mutex<RawFilesStreamer>>);

impl SharedRawFilesStreamer {
    pub fn new(config: RawFilesStreamerConfig) -> Self {
        Self(Arc::new(Mutex::new(RawFilesStreamer::new(config))))
    }

    pub async fn max_size(&self) -> u64 {
        let streamer = self.0.lock().await;

        streamer.max_size()
    }

    #[allow(unused)]
    pub async fn max_chunk_size(&self) -> u64 {
        let streamer = self.0.lock().await;

        streamer.max_chunk_size()
    }

    pub async fn insert(
        &self,
        path: impl Into<PathBuf>,
        file_name: impl Into<String>,
        size: u64,
    ) -> SourceControlResult<FileId> {
        let mut streamer = self.0.lock().await;

        streamer.insert(path, file_name, size)
    }

    pub async fn contains_key(&self, id: &FileId) -> bool {
        let streamer = self.0.lock().await;

        streamer.contains_key(id)
    }

    pub async fn push(&self, id: &FileId, content: &[u8]) -> SourceControlResult<RawFileStatus> {
        let mut streamer = self.0.lock().await;

        streamer.push(id, content).await
    }

    pub async fn cancel(&self, id: &FileId) -> bool {
        let mut streamer = self.0.lock().await;

        streamer.cancel(id)
    }

    async fn stream(
        &self,
        id: FileId,
        content: Vec<u8>,
    ) -> SourceControlResult<ReceiverStream<RawFileStatus>> {
        let (tx, rx) = mpsc::channel(32);

        let streamer = self.clone();

        tokio::spawn(async move {
            let mut chunks = tokio_stream::iter(
                content.chunks(streamer.max_chunk_size().await.try_into().unwrap()),
            );

            while let Some(content) = chunks.next().await {
                let tx = tx.clone();

                if !streamer.contains_key(&id).await {
                    tx.send(RawFileStatus::Error {
                        error: SourceControlError::UnknownId(id.clone()),
                    })
                    .await
                    .map_err(|_error| "Couldn't send file upload stream error message".to_string())
                    .unwrap();
                } else {
                    let tx = tx.clone();

                    match streamer.push(&id, content).await {
                        Ok(status) => {
                            tx.send(status).await.unwrap();
                        }
                        Err(error) => {
                            tx.send(RawFileStatus::Error { error })
                                .await
                                .map_err(|error| {
                                    format!(
                                        "Couldn't send file upload stream error message: {}",
                                        error
                                    )
                                })
                                .unwrap();
                        }
                    };
                };
            }
        });

        Ok(ReceiverStream::new(rx))
    }
}

impl Default for SharedRawFilesStreamer {
    fn default() -> Self {
        Self::new(RawFilesStreamerConfig::default())
    }
}

impl From<RawFilesStreamer> for SharedRawFilesStreamer {
    fn from(streamer: RawFilesStreamer) -> Self {
        Self::new(streamer.config)
    }
}

impl Clone for SharedRawFilesStreamer {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub(crate) struct SourceControlRPC {
    streamer: SharedRawFilesStreamer,
    uploads_folder: PathBuf,
    transaction_manager: Arc<Mutex<TransactionManager>>,
}

#[derive(Default)]
pub(crate) struct SourceControlPlugin;

impl SourceControlPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn setup(
        transaction_manager: Res<'_, Arc<Mutex<TransactionManager>>>,
        settings: Res<'_, ResourceRegistrySettings>,
        streamer: Res<'_, SharedRawFilesStreamer>,
        mut grpc_settings: ResMut<'_, GRPCPluginSettings>,
    ) {
        let property_inspector = SourceControlServer::new(SourceControlRPC {
            streamer: streamer.clone(),
            uploads_folder: settings.root_folder().join("uploads"),
            transaction_manager: transaction_manager.clone(),
        });

        grpc_settings.register_service(property_inspector);
    }
}

impl Plugin for SourceControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            Self::setup
                .exclusive_system()
                .after(ResourceRegistryPluginScheduling::ResourceRegistryCreated)
                .before(GRPCPluginScheduling::StartRpcServer),
        );
    }
}

#[tonic::async_trait]
impl SourceControl for SourceControlRPC {
    async fn init_upload_raw_file(
        &self,
        request: Request<InitUploadRawFileRequest>,
    ) -> Result<Response<InitUploadRawFileResponse>, Status> {
        let message = request.into_inner();

        let size = message.size;

        if size > self.streamer.max_size().await {
            return Ok(Response::new(InitUploadRawFileResponse {
                status: UploadStatus::Rejected as i32,
                id: None,
            }));
        }

        let file_id = self
            .streamer
            .insert(self.uploads_folder.as_path(), &message.name, size)
            .await
            .map_err(|_error| {
                Status::internal(format!("Couldn't create file for {}", message.name))
            })?;

        Ok(Response::new(InitUploadRawFileResponse {
            status: UploadStatus::Queued as i32,
            id: Some(file_id.into()),
        }))
    }

    type UploadRawFileStream = ReceiverStream<Result<UploadRawFileResponse, Status>>;

    async fn upload_raw_file(
        &self,
        request: Request<UploadRawFileRequest>,
    ) -> Result<Response<Self::UploadRawFileStream>, Status> {
        let message = request.into_inner();

        let streamer = self.streamer.clone();

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            // Unfortunately it's not possible to have bidirectional streaming client -> server -> client
            // so we do receive the whole file and then chunk it and save it progressively.
            // When bidirectional streaming will be achievable we will receive only chunks of files.
            let mut stream = streamer
                .stream(message.id.clone().into(), message.content)
                .await
                .unwrap();

            while let Some(status) = stream.next().await {
                let response = match status {
                    RawFileStatus::Done => Ok(UploadRawFileResponse {
                        response: Some(upload_raw_file_response::Response::Progress(
                            UploadRawFileProgress {
                                id: message.id.clone(),
                                completion: None,
                                status: UploadStatus::Done as i32,
                            },
                        )),
                    }),
                    RawFileStatus::InProgress { completion } => Ok(UploadRawFileResponse {
                        response: Some(upload_raw_file_response::Response::Progress(
                            UploadRawFileProgress {
                                id: message.id.clone(),
                                completion: Some(completion),
                                status: UploadStatus::Started as i32,
                            },
                        )),
                    }),
                    RawFileStatus::Error { error } => Ok(UploadRawFileResponse {
                        response: Some(upload_raw_file_response::Response::Error(
                            error.to_string(),
                        )),
                    }),
                };

                tx.send(response).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn cancel_upload_raw_file(
        &self,
        request: Request<CancelUploadRawFileRequest>,
    ) -> Result<Response<CancelUploadRawFileResponse>, Status> {
        let message = request.into_inner();

        let canceled = self.streamer.cancel(&message.id.into()).await;

        Ok(Response::new(CancelUploadRawFileResponse { ok: canceled }))
    }

    async fn commit_staged_resources(
        &self,
        request: Request<CommitStagedResourcesRequest>,
    ) -> Result<Response<CommitStagedResourcesResponse>, Status> {
        let request = request.into_inner();
        let transaction_manager = self.transaction_manager.lock().await;
        let mut ctx = LockContext::new(&transaction_manager).await;
        ctx.project
            .commit(&request.message)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;
        Ok(Response::new(CommitStagedResourcesResponse {}))
    }

    async fn sync_latest(
        &self,
        request: Request<SyncLatestResquest>,
    ) -> Result<Response<SyncLatestResponse>, Status> {
        let _request = request.into_inner();

        let (resource_to_build, resource_to_unload) = {
            let mut resource_to_build = Vec::new();
            let mut resource_to_unload = Vec::new();

            let transaction_manager = self.transaction_manager.lock().await;
            let mut ctx = LockContext::new(&transaction_manager).await;
            let changes = ctx.project.sync_latest().await.map_err(|err| {
                error!("Failed to get changes: {}", err);
                Status::internal(err.to_string())
            })?;

            for (id, change_type) in changes {
                if let Ok(kind) = ctx.project.resource_type(id) {
                    let resource_id = ResourceTypeAndId { kind, id };
                    match change_type {
                        ChangeType::Add | ChangeType::Edit => {
                            resource_to_build.push(resource_id);
                        }
                        ChangeType::Delete => {
                            resource_to_unload.push(resource_id);
                        }
                    }
                } else {
                    error!("Failed to retrieve resource type for {}", id);
                }
            }
            (resource_to_build, resource_to_unload)
        };

        let transaction_manager = self.transaction_manager.lock().await;
        for resource_id in resource_to_build {
            if resource_id.kind == sample_data::offline::Entity::TYPE {
                {
                    let mut ctx = LockContext::new(&transaction_manager).await;
                    if let Err(err) = ctx.reload(resource_id).await {
                        error!("Failed to reload resource {}: {}", resource_id, err);
                    }
                }
                if let Err(err) = transaction_manager.build_by_id(resource_id).await {
                    error!("Failed to compile resource {}: {}", resource_id, err);
                }
            }
        }

        for resource_id in resource_to_unload {
            let mut ctx = LockContext::new(&transaction_manager).await;
            ctx.unload(resource_id).await;
        }

        Ok(Response::new(SyncLatestResponse {}))
    }

    async fn get_staged_resources(
        &self,
        _request: Request<GetStagedResourcesRequest>,
    ) -> Result<Response<GetStagedResourcesResponse>, Status> {
        /*
        let transaction_manager = self.transaction_manager.lock().await;
        let mut ctx = LockContext::new(&transaction_manager).await;
        let changes = ctx
            .project
            .get_staged_changes()
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let mut entries = Vec::<StagedResource>::new();
        for (resource_id, change_type) in changes {
            let (path, kind) = if let ChangeType::Delete = change_type {
                ctx.project
                    .deleted_resource_info(resource_id)
                    .await
                    .unwrap_or_else(|_err| ("(error)".into(), sample_data::offline::Entity::TYPE))
            } else {
                let path = ctx
                    .project
                    .resource_name(resource_id)
                    .unwrap_or_else(|_err| "(error)".into());

                let kind = ctx
                    .project
                    .resource_type(resource_id)
                    .unwrap_or(sample_data::offline::Entity::TYPE); // Hack, figure out a way to get type for deleted resources
                (path, kind)
            };

            entries.push(StagedResource {
                info: Some(ResourceDescription {
                    id: ResourceTypeAndId::to_string(&ResourceTypeAndId {
                        kind,
                        id: resource_id,
                    }),
                    path: path.to_string(),
                    r#type: kind.as_pretty().trim_start_matches("offline_").into(),
                    version: 1,
                }),
                change_type: match change_type {
                    ChangeType::Add => staged_resource::ChangeType::Add as i32,
                    ChangeType::Edit => staged_resource::ChangeType::Edit as i32,
                    ChangeType::Delete => staged_resource::ChangeType::Delete as i32,
                },
            });
        }

        Ok(Response::new(GetStagedResourcesResponse { entries }))
        */
        Err(Status::internal("todo".to_string()))
    }

    async fn revert_resources(
        &self,
        request: Request<RevertResourcesRequest>,
    ) -> Result<Response<RevertResourcesResponse>, Status> {
        let request = request.into_inner();
        let transaction_manager = self.transaction_manager.lock().await;
        let mut ctx = LockContext::new(&transaction_manager).await;

        let mut need_rebuild = Vec::new();
        for id in &request.ids {
            if let Ok(id) = ResourceTypeAndId::from_str(id) {
                match ctx.project.revert_resource(id.id).await {
                    Ok(()) => need_rebuild.push(id),
                    Err(err) => lgn_tracing::error!("Failed to revert {}: {}", id, err),
                }
            } else {
                error!("Invalid ResourceTypeAndId format {}", id);
            }
        }

        for id in need_rebuild {
            match ctx.build.build_all_derived(id, &ctx.project).await {
                Ok((runtime_path_id, _built_resources)) => {
                    ctx.asset_registry.reload(runtime_path_id.resource_id());
                }
                Err(e) => {
                    lgn_tracing::error!("Error building resource derivations {:?}", e);
                }
            }
        }

        Ok(Response::new(RevertResourcesResponse {}))
    }

    async fn pull_asset(
        &self,
        request: Request<PullAssetRequest>,
    ) -> Result<Response<PullAssetResponse>, Status> {
        let message = request.into_inner();
        let transaction_manager = self.transaction_manager.lock().await;
        let ctx = LockContext::new(&transaction_manager).await;
        let id = ResourceTypeAndId::from_str(message.id.as_str()).map_err(Status::unknown)?;
        if id.kind != GltfFile::TYPE {
            return Err(Status::internal(
                "pull_asset supports GltfFile only at the moment",
            ));
        }
        let resource = ctx.asset_registry.load_sync::<GltfFile>(id);
        if let Some(gltf_file) = resource.get(&ctx.asset_registry) {
            return Ok(Response::new(PullAssetResponse {
                size: gltf_file.bytes().len() as u32,
                content: gltf_file.bytes().to_vec(),
            }));
        }
        return Err(Status::internal(format!("Failed to get an asset {}", id)));
    }
}
