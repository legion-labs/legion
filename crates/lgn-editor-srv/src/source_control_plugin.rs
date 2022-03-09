use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::File,
    io::Write,
    path::PathBuf,
    sync::Arc,
};

use bytes::BytesMut;
use lgn_app::prelude::*;
use lgn_data_runtime::{Resource, ResourceTypeAndId};
use lgn_data_transaction::{LockContext, TransactionManager};
use lgn_ecs::{
    prelude::{IntoExclusiveSystem, Res, ResMut},
    schedule::ExclusiveSystemDescriptorCoercion,
};
use lgn_editor_proto::source_control::{
    source_control_server::{SourceControl, SourceControlServer},
    staged_resource, upload_raw_file_response, CancelUploadRawFileRequest,
    CancelUploadRawFileResponse, CommitStagedResourcesRequest, CommitStagedResourcesResponse,
    GetStagedResourcesRequest, GetStagedResourcesResponse, InitUploadRawFileRequest,
    InitUploadRawFileResponse, ResourceDescription, StagedResource, SyncLatestResponse,
    SyncLatestResquest, UploadRawFileProgress, UploadRawFileRequest, UploadRawFileResponse,
    UploadStatus,
};
use lgn_grpc::{GRPCPluginScheduling, GRPCPluginSettings};
use lgn_resource_registry::{ResourceRegistryPluginScheduling, ResourceRegistrySettings};
use lgn_source_control::ChangeType;
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

impl<S: Into<String>> From<S> for FileId {
    fn from(s: S) -> Self {
        Self(s.into())
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

impl RawFile {
    pub fn new(
        path: impl Into<PathBuf>,
        name: impl Into<String>,
        file_size: usize,
        buffer_size: usize,
    ) -> SourceControlResult<Self> {
        let name = name.into();

        let path = path.into().join(&name);

        let file = File::create(&path)
            .map_err(|error| SourceControlError::FileCreation(path, Box::new(error)))?;

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

#[derive(Default)]
pub(crate) struct SourceControlPlugin;

pub(crate) struct SourceControlRPC {
    streamer: SharedRawFilesStreamer,
    uploads_folder: PathBuf,
    transaction_manager: Arc<Mutex<TransactionManager>>,
}

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

        let valid_name: Cow<'_, str> = if message.name.contains(' ') {
            message.name.replace(" ", "_").into()
        } else {
            message.name.into()
        };

        if size > self.streamer.max_size().await {
            return Ok(Response::new(InitUploadRawFileResponse {
                status: UploadStatus::Rejected as i32,
                id: None,
                name: None,
            }));
        }

        let file_id = self
            .streamer
            .insert(self.uploads_folder.as_path(), &*valid_name, size)
            .await
            .map_err(|_error| {
                Status::internal(format!("Couldn't not create file for {}", valid_name))
            })?;

        Ok(Response::new(InitUploadRawFileResponse {
            status: UploadStatus::Queued as i32,
            id: Some(file_id.to_string()),
            name: Some(valid_name.into_owned()),
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
        let mut transaction_manager = self.transaction_manager.lock().await;
        {
            let mut ctx = LockContext::new(&transaction_manager).await;
            ctx.project
                .sync_latest()
                .await
                .map_err(|err| Status::internal(err.to_string()))?;
        }

        transaction_manager.load_all_resources().await;

        Ok(Response::new(SyncLatestResponse {}))
    }

    async fn get_staged_resources(
        &self,
        _request: Request<GetStagedResourcesRequest>,
    ) -> Result<Response<GetStagedResourcesResponse>, Status> {
        let transaction_manager = self.transaction_manager.lock().await;
        let ctx = LockContext::new(&transaction_manager).await;
        let changes = ctx
            .project
            .get_staged_changes()
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let entries: Vec<StagedResource> = changes
            .into_iter()
            .map(|(resource_id, change)| {
                let path: String = ctx
                    .project
                    .resource_name(resource_id)
                    .unwrap_or_else(|_err| "(deleted)".into())
                    .to_string();

                let kind = ctx
                    .project
                    .resource_type(resource_id)
                    .unwrap_or(sample_data::offline::Entity::TYPE); // Hack, figure out a way to get type for deleted resources
                StagedResource {
                    info: Some(ResourceDescription {
                        id: ResourceTypeAndId::to_string(&ResourceTypeAndId {
                            kind,
                            id: resource_id,
                        }),
                        path,
                        r#type: kind.as_pretty().trim_start_matches("offline_").into(),
                        version: 1,
                    }),
                    change_type: match change.change_type() {
                        ChangeType::Add { .. } => staged_resource::ChangeType::Add as i32,
                        ChangeType::Edit { .. } => staged_resource::ChangeType::Edit as i32,
                        ChangeType::Delete { .. } => staged_resource::ChangeType::Delete as i32,
                    },
                }
            })
            .collect();

        Ok(Response::new(GetStagedResourcesResponse { entries }))
    }
}
