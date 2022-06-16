use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use async_trait::async_trait;
use bytes::BytesMut;
use editor_srv::source_control::{
    server::{
        register_routes, CommitStagedResourcesRequest, CommitStagedResourcesResponse,
        ContentUploadCancelRequest, ContentUploadCancelResponse, ContentUploadInitRequest,
        ContentUploadInitResponse, ContentUploadRequest, ContentUploadResponse,
        GetStagedResourcesRequest, GetStagedResourcesResponse, PullAssetsRequest,
        PullAssetsResponse, RevertResourcesRequest, RevertResourcesResponse, SyncLatestRequest,
        SyncLatestResponse,
    },
    Api, ContentUploadInitSucceeded, ContentUploadSucceeded, ContentUploadSucceededStatus,
    PullAssetsSucceeded, ResourceDescription, StagedResource, StagedResourceChangeType,
    StagedResources,
};
use lgn_app::prelude::*;
use lgn_data_offline::resource::ChangeType;
use lgn_data_runtime::{ResourceDescriptor, ResourceTypeAndId};
use lgn_data_transaction::{LockContext, TransactionManager};
use lgn_graphics_data::offline_gltf::GltfFile;
use lgn_grpc::SharedRouter;
use lgn_online::server::{Error, Result};
use lgn_resource_registry::ResourceRegistrySettings;
use lgn_tracing::error;
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::codegen::StdError;
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

        if content_len > max_chunk_size {
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
                                        "couldn't send file upload stream error message: {}",
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

impl Plugin for SourceControlPlugin {
    fn build(&self, app: &mut App) {
        let transaction_manager = app
            .world
            .resource::<Arc<Mutex<TransactionManager>>>()
            .clone();
        let settings = app
            .world
            .resource::<ResourceRegistrySettings>()
            .root_folder()
            .join("uploads");
        let streamer = app.world.resource::<SharedRawFilesStreamer>().clone();

        let mut router = app.world.resource_mut::<SharedRouter>();

        let server = Arc::new(Server::new(streamer, settings, transaction_manager));

        router.register_routes(register_routes, server);

        // TODO: Adapt the before/after logic to OpenAPI
        // app.add_startup_system_to_stage(
        //     StartupStage::PostStartup,
        //     Self::setup
        //         .exclusive_system()
        //         .after(ResourceRegistryPluginScheduling::ResourceRegistryCreated)
        //         .before(GRPCPluginScheduling::StartRpcServer),
        // );
    }
}

pub(crate) struct Server {
    streamer: SharedRawFilesStreamer,
    uploads_folder: PathBuf,
    #[allow(dead_code)]
    transaction_manager: Arc<Mutex<TransactionManager>>,
}

impl Server {
    pub(crate) fn new(
        streamer: SharedRawFilesStreamer,
        uploads_folder: PathBuf,
        transaction_manager: Arc<Mutex<TransactionManager>>,
    ) -> Self {
        Self {
            streamer,
            uploads_folder,
            transaction_manager,
        }
    }
}

#[async_trait]
impl Api for Server {
    async fn content_upload_init(
        &self,
        request: ContentUploadInitRequest,
    ) -> Result<ContentUploadInitResponse> {
        if request.body.size > self.streamer.max_size().await {
            return Ok(ContentUploadInitResponse::Status400);
        }

        let file_id = self
            .streamer
            .insert(
                self.uploads_folder.as_path(),
                &request.body.name,
                request.body.size,
            )
            .await
            .map_err(|_error| {
                Error::internal(format!("couldn't create file for {}", request.body.name))
            })?;

        Ok(ContentUploadInitResponse::Status200(
            ContentUploadInitSucceeded { id: file_id.into() },
        ))
    }

    async fn content_upload(&self, request: ContentUploadRequest) -> Result<ContentUploadResponse> {
        let streamer = self.streamer.clone();

        // Unfortunately it's not possible to have bidirectional streaming client -> server -> client
        // so we do receive the whole file and then chunk it and save it progressively.
        // When bidirectional streaming will be achievable we will receive only chunks of files.
        let mut stream = streamer
            .stream(request.transaction_id.0.clone().into(), request.body.0)
            .await
            .unwrap();

        let mut last_status = None;

        while let Some(status) = stream.next().await {
            match status {
                RawFileStatus::Error {
                    error: SourceControlError::UnknownId(_),
                } => {
                    return Ok(ContentUploadResponse::Status404);
                }
                RawFileStatus::Error {
                    error: SourceControlError::ContentSize { .. },
                } => {
                    return Ok(ContentUploadResponse::Status400);
                }
                RawFileStatus::Error { error } => {
                    return Err(Error::internal(error.to_string()));
                }
                RawFileStatus::InProgress { completion } => {
                    last_status = Some((
                        ContentUploadSucceededStatus::InProgress,
                        completion.round() as u32,
                    ));
                }
                RawFileStatus::Done => {
                    last_status = Some((ContentUploadSucceededStatus::Done, 100));
                }
            };
        }

        if let Some((status, completion)) = last_status {
            Ok(ContentUploadResponse::Status200(ContentUploadSucceeded {
                id: request.transaction_id.0,
                status,
                completion,
            }))
        } else {
            Err(Error::internal(
                "didn't receive any status from upload stream",
            ))
        }
    }

    async fn content_upload_cancel(
        &self,
        request: ContentUploadCancelRequest,
    ) -> Result<ContentUploadCancelResponse> {
        let canceled = self.streamer.cancel(&request.transaction_id.0.into()).await;

        Ok(if canceled {
            ContentUploadCancelResponse::Status204
        } else {
            ContentUploadCancelResponse::Status404
        })
    }

    async fn get_staged_resources(
        &self,
        _request: GetStagedResourcesRequest,
    ) -> Result<GetStagedResourcesResponse> {
        let transaction_manager = self.transaction_manager.lock().await;
        let mut ctx = LockContext::new(&transaction_manager).await;
        let changes = ctx
            .project
            .get_pending_changes()
            .await
            .map_err(|err| Error::internal(err.to_string()))?;

        let mut entries = Vec::new();

        for (resource_type_id, change_type) in changes {
            let path = if let ChangeType::Delete = change_type {
                ctx.project
                    .deleted_resource_info(resource_type_id)
                    .await
                    .unwrap_or_else(|_err| "(error)".into())
            } else {
                ctx.project
                    .resource_name(resource_type_id)
                    .await
                    .unwrap_or_else(|_err| "(error)".into())
            };

            entries.push(StagedResource {
                info: ResourceDescription {
                    id: ResourceTypeAndId::to_string(&resource_type_id),
                    path: path.to_string(),
                    type_: resource_type_id
                        .kind
                        .as_pretty()
                        .trim_start_matches("offline_")
                        .into(),
                    version: 1,
                },
                change_type: match change_type {
                    ChangeType::Add => StagedResourceChangeType::Add,
                    ChangeType::Edit => StagedResourceChangeType::Edit,
                    ChangeType::Delete => StagedResourceChangeType::Delete,
                },
            });
        }

        Ok(GetStagedResourcesResponse::Status200(StagedResources(
            entries,
        )))
    }

    async fn commit_staged_resources(
        &self,
        request: CommitStagedResourcesRequest,
    ) -> Result<CommitStagedResourcesResponse> {
        let transaction_manager = self.transaction_manager.lock().await;
        let mut ctx = LockContext::new(&transaction_manager).await;

        ctx.project
            .commit(&request.body.message)
            .await
            .map_err(|err| Error::internal(err.to_string()))?;

        Ok(CommitStagedResourcesResponse::Status204)
    }

    async fn sync_latest(&self, _request: SyncLatestRequest) -> Result<SyncLatestResponse> {
        let (resource_to_build, resource_to_unload) = {
            let mut resource_to_build = Vec::new();
            let mut resource_to_unload = Vec::new();

            let transaction_manager = self.transaction_manager.lock().await;
            let mut ctx = LockContext::new(&transaction_manager).await;
            let changes = ctx.project.sync_latest().await.map_err(|err| {
                error!("Failed to get changes: {}", err);
                Error::internal(err.to_string())
            })?;

            for (resource_id, change_type) in changes {
                match change_type {
                    ChangeType::Add | ChangeType::Edit => {
                        resource_to_build.push(resource_id);
                    }
                    ChangeType::Delete => {
                        resource_to_unload.push(resource_id);
                    }
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

        Ok(SyncLatestResponse::Status204)
    }

    async fn revert_resources(
        &self,
        request: RevertResourcesRequest,
    ) -> Result<RevertResourcesResponse> {
        let transaction_manager = self.transaction_manager.lock().await;
        let mut ctx = LockContext::new(&transaction_manager).await;

        let mut need_rebuild = Vec::new();

        for id in &request.body.0 {
            if let Ok(id) = ResourceTypeAndId::from_str(id) {
                match ctx.project.revert_resource(id).await {
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

        Ok(RevertResourcesResponse::Status204)
    }

    async fn pull_assets(&self, request: PullAssetsRequest) -> Result<PullAssetsResponse> {
        let transaction_manager = self.transaction_manager.lock().await;
        let ctx = LockContext::new(&transaction_manager).await;
        let id = match ResourceTypeAndId::from_str(request.property_id.0.as_str()) {
            Err(_err) => {
                return Ok(PullAssetsResponse::Status400);
            }
            Ok(id) => id,
        };

        if id.kind != GltfFile::TYPE {
            return Err(Error::internal(
                "pull_asset supports GltfFile only at the moment",
            ));
        }

        let resource = ctx.asset_registry.load_sync::<GltfFile>(id);

        if let Some(gltf_file) = resource.get(&ctx.asset_registry) {
            return Ok(PullAssetsResponse::Status200(PullAssetsSucceeded {
                size: gltf_file.bytes().len() as u32,
                content: gltf_file.bytes().to_vec().into(),
            }));
        }

        return Ok(PullAssetsResponse::Status404);
    }
}
