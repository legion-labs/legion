//! Legion Source Control Server
//!
//! TODO: write documentation.

// crate-specific lint exceptions:
//#![allow()]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use lgn_source_control::{new_index_backend, Error};
use lgn_source_control::{BlobStorageUrl, Commit, IndexBackend};
use lgn_source_control_proto::source_control_server::{SourceControl, SourceControlServer};
use lgn_source_control_proto::{
    ClearLockRequest, ClearLockResponse, CommitExistsRequest, CommitExistsResponse,
    CommitToBranchRequest, CommitToBranchResponse, CountLocksInDomainRequest,
    CountLocksInDomainResponse, CreateIndexRequest, CreateIndexResponse, DestroyIndexRequest,
    DestroyIndexResponse, FindBranchRequest, FindBranchResponse, FindBranchesInLockDomainRequest,
    FindBranchesInLockDomainResponse, FindLockRequest, FindLockResponse, FindLocksInDomainRequest,
    FindLocksInDomainResponse, GetBlobStorageUrlRequest, GetBlobStorageUrlResponse,
    IndexExistsRequest, IndexExistsResponse, InsertBranchRequest, InsertBranchResponse,
    InsertCommitRequest, InsertCommitResponse, InsertLockRequest, InsertLockResponse,
    ReadBranchesRequest, ReadBranchesResponse, ReadCommitRequest, ReadCommitResponse,
    ReadTreeRequest, ReadTreeResponse, RegisterWorkspaceRequest, RegisterWorkspaceResponse,
    SaveTreeRequest, SaveTreeResponse, UpdateBranchRequest, UpdateBranchResponse,
};
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::{debug, info, warn, LevelFilter};
use tokio::sync::Mutex;
use url::Url;

struct Service {
    index_backends: Mutex<HashMap<String, Arc<Box<dyn IndexBackend>>>>,
    database_host: String,
    database_username: Option<String>,
    database_password: Option<String>,
    blob_storage_url: BlobStorageUrl,
}

impl Service {
    pub fn new(
        database_host: String,
        database_username: Option<String>,
        database_password: Option<String>,
        blob_storage_url: BlobStorageUrl,
    ) -> Self {
        Self {
            index_backends: Mutex::new(HashMap::new()),
            database_host,
            database_username,
            database_password,
            blob_storage_url,
        }
    }

    fn new_index_backend_for_repository(&self, name: &str) -> Result<Box<dyn IndexBackend>> {
        let index_url = Url::parse_with_params(
            &format!(
                "mysql://{}:{}@{}/{}",
                self.database_username.as_deref().unwrap_or_default(),
                self.database_password.as_deref().unwrap_or_default(),
                self.database_host,
                name,
            ),
            &[("blob_storage_url", self.blob_storage_url.to_string())],
        )
        .unwrap();

        new_index_backend(index_url.as_str()).map_err(Into::into)
    }

    async fn get_index_backend_for_repository(
        &self,
        name: &str,
    ) -> Result<Arc<Box<dyn IndexBackend>>, tonic::Status> {
        let mut index_backends = self.index_backends.lock().await;

        if let Some(index_backend) = index_backends.get(name) {
            Ok(Arc::clone(index_backend))
        } else {
            let backend = Arc::new(
                self.new_index_backend_for_repository(name)
                    .map_err(|e| tonic::Status::unknown(e.to_string()))?,
            );

            index_backends.insert(name.to_string(), backend.clone());

            Ok(backend)
        }
    }

    fn get_request_origin<T>(request: &tonic::Request<T>) -> String {
        request
            .remote_addr()
            .map_or_else(|| "unknown".to_string(), |addr| addr.to_string())
    }
}

#[tonic::async_trait]
impl SourceControl for Service {
    async fn create_index(
        &self,
        request: tonic::Request<CreateIndexRequest>,
    ) -> Result<tonic::Response<CreateIndexResponse>, tonic::Status> {
        let origin = Self::get_request_origin(&request);
        let name = request.into_inner().repository_name;

        debug!("{}: Creating index `{}`...", origin, &name);

        let index_backend = self.get_index_backend_for_repository(&name).await?;

        let blob_storage_url = match index_backend.create_index().await {
            Ok(blob_storage_url) => blob_storage_url,
            Err(Error::IndexAlreadyExists { url: _ }) => {
                warn!(
                    "{}: Did not create index `{}` as it already exists",
                    origin, &name
                );
                return Ok(tonic::Response::new(CreateIndexResponse {
                    blob_storage_url: "".to_string(),
                    already_exists: true,
                }));
            }
            Err(e) => return Err(tonic::Status::unknown(e.to_string())),
        };

        info!("{}: Created index `{}`", origin, &name);

        Ok(tonic::Response::new(CreateIndexResponse {
            blob_storage_url: blob_storage_url.to_string(),
            already_exists: false,
        }))
    }

    async fn destroy_index(
        &self,
        request: tonic::Request<DestroyIndexRequest>,
    ) -> Result<tonic::Response<DestroyIndexResponse>, tonic::Status> {
        let origin = Self::get_request_origin(&request);
        let name = request.into_inner().repository_name;

        // This does not protect from the race condition where a repository is
        // destroyed while another with the same name is being created.
        //
        // We could try to be smart and protect that case by holding the write
        // mutex a little longer, but that would not protect us against the case
        // where several instances of the service are running at the same time,
        // each we a distinct mutex.
        //
        // The only real protection here, is ensuring that this can't happen at
        // the database level, which we don't care about at this early stage.
        let index_backend = self.index_backends.lock().await.remove(&name);

        let index_backend = match index_backend {
            Some(index_backend) => index_backend,
            None => Arc::new(
                self.new_index_backend_for_repository(&name)
                    .map_err(|e| tonic::Status::unknown(e.to_string()))?,
            ),
        };

        match index_backend.destroy_index().await {
            Ok(()) => {
                info!("{}: Destroyed index `{}`", origin, &name);

                Ok(tonic::Response::new(DestroyIndexResponse {
                    does_not_exist: false,
                }))
            }
            Err(Error::IndexDoesNotExist { url: _ }) => {
                warn!(
                    "{}: Did not destroy index `{}` as it does not exist",
                    origin, &name
                );
                Ok(tonic::Response::new(DestroyIndexResponse {
                    does_not_exist: true,
                }))
            }
            Err(e) => Err(tonic::Status::unknown(e.to_string())),
        }
    }

    async fn index_exists(
        &self,
        request: tonic::Request<IndexExistsRequest>,
    ) -> Result<tonic::Response<IndexExistsResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;
        let exists = index_backend
            .index_exists()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(IndexExistsResponse { exists }))
    }

    async fn get_blob_storage_url(
        &self,
        _request: tonic::Request<GetBlobStorageUrlRequest>,
    ) -> Result<tonic::Response<GetBlobStorageUrlResponse>, tonic::Status> {
        Ok(tonic::Response::new(GetBlobStorageUrlResponse {
            blob_storage_url: self.blob_storage_url.to_string(),
        }))
    }

    async fn register_workspace(
        &self,
        request: tonic::Request<RegisterWorkspaceRequest>,
    ) -> Result<tonic::Response<RegisterWorkspaceResponse>, tonic::Status> {
        let request = request.into_inner();
        let workspace_registration = request.workspace_registration.unwrap_or_default().into();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        index_backend
            .register_workspace(&workspace_registration)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(RegisterWorkspaceResponse {}))
    }

    async fn find_branch(
        &self,
        request: tonic::Request<FindBranchRequest>,
    ) -> Result<tonic::Response<FindBranchResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let branch = index_backend
            .find_branch(&request.branch_name)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?
            .map(Into::into);

        Ok(tonic::Response::new(FindBranchResponse { branch }))
    }

    async fn read_branches(
        &self,
        request: tonic::Request<ReadBranchesRequest>,
    ) -> Result<tonic::Response<ReadBranchesResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let branches = index_backend
            .read_branches()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(ReadBranchesResponse {
            branches: branches.into_iter().map(Into::into).collect(),
        }))
    }

    async fn find_branches_in_lock_domain(
        &self,
        request: tonic::Request<FindBranchesInLockDomainRequest>,
    ) -> Result<tonic::Response<FindBranchesInLockDomainResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let branches = index_backend
            .find_branches_in_lock_domain(&request.lock_domain_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(FindBranchesInLockDomainResponse {
            branches: branches.into_iter().map(Into::into).collect(),
        }))
    }

    async fn read_commit(
        &self,
        request: tonic::Request<ReadCommitRequest>,
    ) -> Result<tonic::Response<ReadCommitResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let commit = Some(
            index_backend
                .read_commit(&request.commit_id)
                .await
                .map_err(|e| tonic::Status::unknown(e.to_string()))?
                .into(),
        );

        Ok(tonic::Response::new(ReadCommitResponse { commit }))
    }

    async fn read_tree(
        &self,
        request: tonic::Request<ReadTreeRequest>,
    ) -> Result<tonic::Response<ReadTreeResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let tree = Some(
            index_backend
                .read_tree(&request.tree_hash)
                .await
                .map_err(|e| tonic::Status::unknown(e.to_string()))?
                .into(),
        );

        Ok(tonic::Response::new(ReadTreeResponse { tree }))
    }

    async fn insert_lock(
        &self,
        request: tonic::Request<InsertLockRequest>,
    ) -> Result<tonic::Response<InsertLockResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        index_backend
            .insert_lock(&request.lock.unwrap_or_default().into())
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(InsertLockResponse {}))
    }

    async fn find_lock(
        &self,
        request: tonic::Request<FindLockRequest>,
    ) -> Result<tonic::Response<FindLockResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let lock = index_backend
            .find_lock(&request.lock_domain_id, &request.canonical_relative_path)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?
            .map(Into::into);

        Ok(tonic::Response::new(FindLockResponse { lock }))
    }

    async fn find_locks_in_domain(
        &self,
        request: tonic::Request<FindLocksInDomainRequest>,
    ) -> Result<tonic::Response<FindLocksInDomainResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let locks = index_backend
            .find_locks_in_domain(&request.lock_domain_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(tonic::Response::new(FindLocksInDomainResponse { locks }))
    }

    async fn save_tree(
        &self,
        request: tonic::Request<SaveTreeRequest>,
    ) -> Result<tonic::Response<SaveTreeResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        index_backend
            .save_tree(&request.tree.unwrap_or_default().into(), &request.hash)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(SaveTreeResponse {}))
    }

    async fn insert_commit(
        &self,
        request: tonic::Request<InsertCommitRequest>,
    ) -> Result<tonic::Response<InsertCommitResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let commit: Result<Commit> = request.commit.unwrap_or_default().try_into();
        let commit = commit.map_err(|e| tonic::Status::unknown(e.to_string()))?;

        index_backend
            .insert_commit(&commit)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(InsertCommitResponse {}))
    }

    async fn commit_to_branch(
        &self,
        request: tonic::Request<CommitToBranchRequest>,
    ) -> Result<tonic::Response<CommitToBranchResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let commit: Result<Commit> = request.commit.unwrap_or_default().try_into();
        let commit = commit.map_err(|e| tonic::Status::unknown(e.to_string()))?;

        index_backend
            .commit_to_branch(&commit, &request.branch.unwrap_or_default().into())
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(CommitToBranchResponse {}))
    }

    async fn commit_exists(
        &self,
        request: tonic::Request<CommitExistsRequest>,
    ) -> Result<tonic::Response<CommitExistsResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let exists = index_backend
            .commit_exists(&request.commit_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(CommitExistsResponse { exists }))
    }

    async fn update_branch(
        &self,
        request: tonic::Request<UpdateBranchRequest>,
    ) -> Result<tonic::Response<UpdateBranchResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        index_backend
            .update_branch(&request.branch.unwrap_or_default().into())
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(UpdateBranchResponse {}))
    }

    async fn insert_branch(
        &self,
        request: tonic::Request<InsertBranchRequest>,
    ) -> Result<tonic::Response<InsertBranchResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        index_backend
            .insert_branch(&request.branch.unwrap_or_default().into())
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(InsertBranchResponse {}))
    }

    async fn clear_lock(
        &self,
        request: tonic::Request<ClearLockRequest>,
    ) -> Result<tonic::Response<ClearLockResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        index_backend
            .clear_lock(&request.lock_domain_id, &request.canonical_relative_path)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(ClearLockResponse {}))
    }

    async fn count_locks_in_domain(
        &self,
        request: tonic::Request<CountLocksInDomainRequest>,
    ) -> Result<tonic::Response<CountLocksInDomainResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let count = index_backend
            .count_locks_in_domain(&request.lock_domain_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(CountLocksInDomainResponse { count }))
    }
}

#[derive(Parser, Debug)]
#[clap(name = "Legion Labs source-control server")]
#[clap(about = "Source-control server.", version, author)]
struct Args {
    #[clap(name = "debug", short, long, help = "Enable debug logging")]
    debug: bool,

    /// The address to listen on.
    #[clap(long, default_value = "[::1]:50051")]
    listen_endpoint: SocketAddr,

    /// The SQL database host.
    #[clap(long)]
    database_host: String,

    /// The SQL database username.
    #[clap(long)]
    database_username: Option<String>,

    /// The SQL database password.
    #[clap(long)]
    database_password: Option<String>,

    /// The blob storage URL.
    #[clap(long)]
    blob_storage_url: BlobStorageUrl,
}

#[allow(clippy::semicolon_if_nothing_returned)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let telemetry_guard = TelemetryGuard::default().unwrap();

    let args = Args::parse();

    let _telemetry_guard = if args.debug {
        telemetry_guard.with_log_level(LevelFilter::Debug)
    } else {
        telemetry_guard
    };

    let service = SourceControlServer::new(Service::new(
        args.database_host,
        args.database_username,
        args.database_password,
        args.blob_storage_url,
    ));
    let server = tonic::transport::Server::builder()
        .accept_http1(true)
        .add_service(service);

    server.serve(args.listen_endpoint).await.map_err(Into::into)
}
