//! Legion Source Control Server
//!
//! TODO: write documentation.

// crate-specific lint exceptions:
//#![allow()]

use std::collections::HashMap;
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use lgn_source_control::{
    new_index_backend, CanonicalPath, Commit, CommitId, ContentStoreAddr, Error, IndexBackend,
    ListBranchesQuery, ListCommitsQuery, ListLocksQuery, Lock, Result, Tree,
};
use lgn_source_control_proto::source_control_server::{SourceControl, SourceControlServer};
use lgn_source_control_proto::{
    CommitToBranchRequest, CommitToBranchResponse, CountLocksRequest, CountLocksResponse,
    CreateIndexRequest, CreateIndexResponse, DestroyIndexRequest, DestroyIndexResponse,
    GetBranchRequest, GetBranchResponse, GetLockRequest, GetLockResponse, GetTreeRequest,
    GetTreeResponse, IndexExistsRequest, IndexExistsResponse, InsertBranchRequest,
    InsertBranchResponse, ListBranchesRequest, ListBranchesResponse, ListCommitsRequest,
    ListCommitsResponse, ListLocksRequest, ListLocksResponse, LockRequest, LockResponse,
    RegisterWorkspaceRequest, RegisterWorkspaceResponse, SaveTreeRequest, SaveTreeResponse,
    UnlockRequest, UnlockResponse, UpdateBranchRequest, UpdateBranchResponse,
};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{debug, info, warn, LevelFilter};
use serde::Deserialize;
use tokio::sync::Mutex;
use url::Url;

struct Service {
    index_backends: Mutex<HashMap<String, Arc<Box<dyn IndexBackend>>>>,
    config: Config,
}

impl Service {
    pub fn new(config: Config) -> Self {
        Self {
            index_backends: Mutex::new(HashMap::new()),
            config,
        }
    }

    fn database_name(&self, repository_name: &str) -> String {
        format!("{}-{}", self.config.database.name_prefix, repository_name)
    }

    fn new_index_backend_for_repository(
        &self,
        repository_name: &str,
    ) -> Result<Box<dyn IndexBackend>> {
        let index_url = Url::parse(&format!(
            "mysql://{}:{}@{}/{}",
            self.config.database.username.as_deref().unwrap_or_default(),
            self.config.database.password.as_deref().unwrap_or_default(),
            self.config.database.host,
            self.database_name(repository_name),
        ))
        .unwrap();

        new_index_backend(index_url.as_str()).map_err(Into::into)
    }

    async fn get_index_backend_for_repository(
        &self,
        repository_name: &str,
    ) -> Result<Arc<Box<dyn IndexBackend>>, tonic::Status> {
        let mut index_backends = self.index_backends.lock().await;

        if let Some(index_backend) = index_backends.get(repository_name) {
            Ok(Arc::clone(index_backend))
        } else {
            let backend = Arc::new(
                self.new_index_backend_for_repository(repository_name)
                    .map_err(|e| tonic::Status::unknown(e.to_string()))?,
            );

            index_backends.insert(repository_name.to_string(), backend.clone());

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
        let (name, cas_address) = {
            let msg = request.into_inner();
            (msg.repository_name, msg.cas_address)
        };

        debug!(
            "{}: Creating index `{}` with `{}` cas...",
            origin, &name, &cas_address
        );

        let index_backend = self.get_index_backend_for_repository(&name).await?;

        match index_backend
            .create_index(ContentStoreAddr::from(cas_address))
            .await
        {
            Ok(blob_storage_url) => blob_storage_url,
            Err(Error::IndexAlreadyExists { url: _ }) => {
                warn!(
                    "{}: Did not create index `{}` as it already exists",
                    origin, &name
                );
                return Ok(tonic::Response::new(CreateIndexResponse {
                    already_exists: true,
                }));
            }
            Err(e) => return Err(tonic::Status::unknown(e.to_string())),
        };

        info!("{}: Created index `{}`", origin, &name);

        Ok(tonic::Response::new(CreateIndexResponse {
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

    async fn register_workspace(
        &self,
        request: tonic::Request<RegisterWorkspaceRequest>,
    ) -> Result<tonic::Response<RegisterWorkspaceResponse>, tonic::Status> {
        let request = request.into_inner();
        let workspace_registration = request.workspace_registration.unwrap_or_default().into();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let content_store_addr = index_backend
            .register_workspace(&workspace_registration)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(RegisterWorkspaceResponse {
            content_store_addr: content_store_addr.into(),
        }))
    }

    async fn get_branch(
        &self,
        request: tonic::Request<GetBranchRequest>,
    ) -> Result<tonic::Response<GetBranchResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let branch = match index_backend.get_branch(&request.branch_name).await {
            Ok(branch) => Some(branch.into()),
            Err(Error::BranchNotFound { .. }) => None,
            Err(err) => return Err(tonic::Status::unknown(err.to_string())),
        };

        Ok(tonic::Response::new(GetBranchResponse { branch }))
    }

    async fn list_branches(
        &self,
        request: tonic::Request<ListBranchesRequest>,
    ) -> Result<tonic::Response<ListBranchesResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let query = ListBranchesQuery {
            lock_domain_id: Some(request.lock_domain_id.as_str()).filter(|s| !s.is_empty()),
        };

        let branches = index_backend
            .list_branches(&query)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(ListBranchesResponse {
            branches: branches.into_iter().map(Into::into).collect(),
        }))
    }

    async fn list_commits(
        &self,
        request: tonic::Request<ListCommitsRequest>,
    ) -> Result<tonic::Response<ListCommitsResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let query = ListCommitsQuery {
            commit_ids: request.commit_ids.into_iter().map(CommitId).collect(),
            depth: request.depth,
        };

        let commits = index_backend
            .list_commits(&query)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(tonic::Response::new(ListCommitsResponse { commits }))
    }

    async fn get_tree(
        &self,
        request: tonic::Request<GetTreeRequest>,
    ) -> Result<tonic::Response<GetTreeResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let tree = Some(
            index_backend
                .get_tree(&request.tree_id)
                .await
                .map_err(|e| tonic::Status::unknown(e.to_string()))?
                .into(),
        );

        Ok(tonic::Response::new(GetTreeResponse { tree }))
    }

    async fn lock(
        &self,
        request: tonic::Request<LockRequest>,
    ) -> Result<tonic::Response<LockResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let lock: Result<Lock> = request.lock.unwrap_or_default().try_into();
        let lock = lock.map_err(|e| tonic::Status::unknown(e.to_string()))?;

        index_backend
            .lock(&lock)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(LockResponse {}))
    }

    async fn get_lock(
        &self,
        request: tonic::Request<GetLockRequest>,
    ) -> Result<tonic::Response<GetLockResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let lock = match index_backend
            .get_lock(
                &request.lock_domain_id,
                &CanonicalPath::new(&request.canonical_path)
                    .map_err(|e| tonic::Status::unknown(e.to_string()))?,
            )
            .await
        {
            Ok(lock) => Some(lock.into()),
            Err(Error::LockNotFound { .. }) => None,
            Err(err) => return Err(tonic::Status::unknown(err.to_string())),
        };

        Ok(tonic::Response::new(GetLockResponse { lock }))
    }

    async fn list_locks(
        &self,
        request: tonic::Request<ListLocksRequest>,
    ) -> Result<tonic::Response<ListLocksResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let query = ListLocksQuery {
            lock_domain_ids: request.lock_domain_ids.iter().map(String::as_str).collect(),
        };

        let locks = index_backend
            .list_locks(&query)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(tonic::Response::new(ListLocksResponse { locks }))
    }

    async fn save_tree(
        &self,
        request: tonic::Request<SaveTreeRequest>,
    ) -> Result<tonic::Response<SaveTreeResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let tree: Result<Tree> = request.tree.unwrap_or_default().try_into();
        let tree = tree.map_err(|e| tonic::Status::unknown(e.to_string()))?;

        let tree_id = index_backend
            .save_tree(&tree)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(SaveTreeResponse { tree_id }))
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

        let commit_id = index_backend
            .commit_to_branch(&commit, &request.branch.unwrap_or_default().into())
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(CommitToBranchResponse {
            commit_id: commit_id.0,
        }))
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

    async fn unlock(
        &self,
        request: tonic::Request<UnlockRequest>,
    ) -> Result<tonic::Response<UnlockResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        index_backend
            .unlock(
                &request.lock_domain_id,
                &CanonicalPath::new(&request.canonical_path)
                    .map_err(|e| tonic::Status::unknown(e.to_string()))?,
            )
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(UnlockResponse {}))
    }

    async fn count_locks(
        &self,
        request: tonic::Request<CountLocksRequest>,
    ) -> Result<tonic::Response<CountLocksResponse>, tonic::Status> {
        let request = request.into_inner();
        let index_backend = self
            .get_index_backend_for_repository(&request.repository_name)
            .await?;

        let query = ListLocksQuery {
            lock_domain_ids: request.lock_domain_ids.iter().map(String::as_str).collect(),
        };

        let count = index_backend
            .count_locks(&query)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(CountLocksResponse { count }))
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
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct DatabaseConfig {
    /// The database host.
    host: String,

    /// The database name prefix.
    #[serde(default = "DatabaseConfig::default_name_prefix")]
    name_prefix: String,

    // The database username.
    username: Option<String>,

    /// The database password.
    password: Option<String>,
}

impl DatabaseConfig {
    fn default_name_prefix() -> String {
        "source_control".to_string()
    }
}

impl Display for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(username) = &self.username {
            write!(
                f,
                "mysql://{}:**secret**@{}/{}_<repository_name>",
                username, self.host, self.name_prefix,
            )
        } else {
            write!(
                f,
                "mysql://{}/{}_<repository_name>",
                self.host, self.name_prefix,
            )
        }
    }
}

#[allow(clippy::semicolon_if_nothing_returned)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let _telemetry_guard = if args.debug {
        TelemetryGuardBuilder::default()
            .with_local_sink_max_level(LevelFilter::Debug)
            .build()
    } else {
        TelemetryGuardBuilder::default().build()
    };

    let config: Config = lgn_config::get("source_control.server")?
        .ok_or_else(|| anyhow::anyhow!("no configuration was found for `source-control.server`"))?;

    info!("Using database at {}", config.database);

    let service = SourceControlServer::new(Service::new(config));
    let server = tonic::transport::Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(service));

    info!("Listening on {}", args.listen_endpoint);

    server.serve(args.listen_endpoint).await.map_err(Into::into)
}
