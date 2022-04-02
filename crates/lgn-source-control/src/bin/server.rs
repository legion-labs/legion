//! Legion Source Control Server
//!
//! TODO: write documentation.

// crate-specific lint exceptions:
//#![allow()]

use std::fmt::Display;
use std::net::SocketAddr;
use std::time::Duration;

use clap::Parser;
use http::{header, Method};
use lgn_online::authentication::jwt::RequestAuthorizer;
use lgn_online::authentication::UserInfo;
use lgn_source_control::{
    CanonicalPath, Commit, CommitId, Error, Index, ListBranchesQuery, ListCommitsQuery,
    ListLocksQuery, Lock, RepositoryIndex, RepositoryName, Result, SqlRepositoryIndex, Tree,
};
use lgn_source_control_proto::source_control_server::{SourceControl, SourceControlServer};
use lgn_source_control_proto::{
    CommitToBranchRequest, CommitToBranchResponse, CountLocksRequest, CountLocksResponse,
    CreateRepositoryRequest, CreateRepositoryResponse, DestroyRepositoryRequest,
    DestroyRepositoryResponse, GetBranchRequest, GetBranchResponse, GetLockRequest,
    GetLockResponse, GetTreeRequest, GetTreeResponse, InsertBranchRequest, InsertBranchResponse,
    ListBranchesRequest, ListBranchesResponse, ListCommitsRequest, ListCommitsResponse,
    ListLocksRequest, ListLocksResponse, LockRequest, LockResponse, RegisterWorkspaceRequest,
    RegisterWorkspaceResponse, RepositoryExistsRequest, RepositoryExistsResponse, SaveTreeRequest,
    SaveTreeResponse, UnlockRequest, UnlockResponse, UpdateBranchRequest, UpdateBranchResponse,
};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{debug, info, warn, LevelFilter};
use serde::Deserialize;
use tonic::transport::Server;
use tower_http::{
    auth::RequireAuthorizationLayer,
    cors::{CorsLayer, Origin},
};

struct Service {
    repository_index: SqlRepositoryIndex,
}

impl Service {
    pub fn new(repository_index: SqlRepositoryIndex) -> Self {
        Self { repository_index }
    }

    async fn get_index_for_repository(
        &self,
        repository_name: RepositoryName,
    ) -> Result<Box<dyn Index>, tonic::Status> {
        self.repository_index
            .load_repository(repository_name)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))
    }

    fn get_request_origin<T>(request: &tonic::Request<T>) -> String {
        request
            .remote_addr()
            .map_or_else(|| "unknown".to_string(), |addr| addr.to_string())
    }
}

#[tonic::async_trait]
impl SourceControl for Service {
    async fn create_repository(
        &self,
        request: tonic::Request<CreateRepositoryRequest>,
    ) -> Result<tonic::Response<CreateRepositoryResponse>, tonic::Status> {
        let origin = Self::get_request_origin(&request);
        let repository_name = request
            .into_inner()
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;

        debug!("{}: Creating repository `{}`...", origin, &repository_name);

        match self
            .repository_index
            .create_repository(repository_name)
            .await
        {
            Ok(blob_storage_url) => blob_storage_url,
            Err(Error::RepositoryAlreadyExists { repository_name }) => {
                warn!(
                    "{}: Did not create repository `{}` as it already exists",
                    origin, repository_name
                );

                return Ok(tonic::Response::new(CreateRepositoryResponse {
                    already_exists: true,
                }));
            }
            Err(e) => return Err(tonic::Status::unknown(e.to_string())),
        };

        info!("{}: Created repository.", origin);

        Ok(tonic::Response::new(CreateRepositoryResponse {
            already_exists: false,
        }))
    }

    async fn destroy_repository(
        &self,
        request: tonic::Request<DestroyRepositoryRequest>,
    ) -> Result<tonic::Response<DestroyRepositoryResponse>, tonic::Status> {
        let origin = Self::get_request_origin(&request);
        let repository_name: RepositoryName = request
            .into_inner()
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;

        match self
            .repository_index
            .destroy_repository(repository_name.clone())
            .await
        {
            Ok(()) => {
                info!("{}: Destroyed repository `{}`", origin, &repository_name);

                Ok(tonic::Response::new(DestroyRepositoryResponse {
                    does_not_exist: false,
                }))
            }
            Err(Error::RepositoryDoesNotExist { repository_name: _ }) => {
                warn!(
                    "{}: Did not destroy repository `{}` as it does not exist",
                    origin, &repository_name
                );
                Ok(tonic::Response::new(DestroyRepositoryResponse {
                    does_not_exist: true,
                }))
            }
            Err(e) => Err(tonic::Status::unknown(e.to_string())),
        }
    }

    async fn repository_exists(
        &self,
        request: tonic::Request<RepositoryExistsRequest>,
    ) -> Result<tonic::Response<RepositoryExistsResponse>, tonic::Status> {
        let repository_name = request
            .into_inner()
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;

        match self.repository_index.load_repository(repository_name).await {
            Ok(_) => Ok(tonic::Response::new(RepositoryExistsResponse {
                exists: true,
            })),
            Err(Error::RepositoryDoesNotExist { repository_name: _ }) => {
                Ok(tonic::Response::new(RepositoryExistsResponse {
                    exists: false,
                }))
            }
            Err(e) => Err(tonic::Status::unknown(e.to_string())),
        }
    }

    async fn register_workspace(
        &self,
        request: tonic::Request<RegisterWorkspaceRequest>,
    ) -> Result<tonic::Response<RegisterWorkspaceResponse>, tonic::Status> {
        let request = request.into_inner();
        let workspace_registration = request.workspace_registration.unwrap_or_default().into();
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        index
            .register_workspace(&workspace_registration)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(RegisterWorkspaceResponse {}))
    }

    async fn get_branch(
        &self,
        request: tonic::Request<GetBranchRequest>,
    ) -> Result<tonic::Response<GetBranchResponse>, tonic::Status> {
        let request = request.into_inner();
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let branch = match index.get_branch(&request.branch_name).await {
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let query = ListBranchesQuery {
            lock_domain_id: Some(request.lock_domain_id.as_str()).filter(|s| !s.is_empty()),
        };

        let branches = index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let query = ListCommitsQuery {
            commit_ids: request.commit_ids.into_iter().map(CommitId).collect(),
            depth: request.depth,
        };

        let commits = index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let tree = Some(
            index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let lock: Result<Lock> = request.lock.unwrap_or_default().try_into();
        let lock = lock.map_err(|e| tonic::Status::unknown(e.to_string()))?;

        index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let lock = match index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let query = ListLocksQuery {
            lock_domain_ids: request.lock_domain_ids.iter().map(String::as_str).collect(),
        };

        let locks = index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let tree: Result<Tree> = request.tree.unwrap_or_default().try_into();
        let tree = tree.map_err(|e| tonic::Status::unknown(e.to_string()))?;

        let tree_id = index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let commit: Result<Commit> = request.commit.unwrap_or_default().try_into();
        let commit = commit.map_err(|e| tonic::Status::unknown(e.to_string()))?;

        let commit_id = index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        index
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
        let repository_name = request
            .repository_name
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        let index = self.get_index_for_repository(repository_name).await?;

        let query = ListLocksQuery {
            lock_domain_ids: request.lock_domain_ids.iter().map(String::as_str).collect(),
        };

        let count = index
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

    #[clap(
        long,
        default_value = "",
        help = "The list of origins that are allowed to make requests, for CORS"
    )]
    origins: Vec<http::HeaderValue>,
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
    #[serde(default = "DatabaseConfig::default_name")]
    name: String,

    // The database username.
    username: Option<String>,

    /// The database password.
    password: Option<String>,
}

impl DatabaseConfig {
    fn default_name() -> String {
        "source_control".to_string()
    }

    async fn instantiate_repository_index(&self) -> Result<SqlRepositoryIndex> {
        let index_url = format!(
            "mysql://{}:{}@{}/{}",
            self.username.as_deref().unwrap_or_default(),
            self.password.as_deref().unwrap_or_default(),
            self.host,
            self.name,
        );

        SqlRepositoryIndex::new(index_url).await
    }
}

impl Display for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(username) = &self.username {
            write!(
                f,
                "mysql://{}:**secret**@{}/{}",
                username, self.host, self.name,
            )
        } else {
            write!(f, "mysql://{}/{}", self.host, self.name,)
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

    let cors = CorsLayer::new()
        .allow_origin(Origin::list(args.origins))
        .allow_credentials(true)
        .max_age(Duration::from_secs(60 * 60))
        .allow_headers(vec![
            header::ACCEPT,
            header::ACCEPT_LANGUAGE,
            header::AUTHORIZATION,
            header::CONTENT_LANGUAGE,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-grpc-web"),
        ])
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::HEAD,
            Method::OPTIONS,
            Method::CONNECT,
        ])
        .expose_headers(tower_http::cors::Any {});

    let validation = lgn_online::Config::load()?
        .signature_validation
        .instantiate_validation()
        .await?;

    let auth_layer =
        RequireAuthorizationLayer::custom(RequestAuthorizer::<UserInfo, _, _>::new(validation));

    let layer = tower::ServiceBuilder::new() //todo: compose with cors layer
        .layer(auth_layer)
        .layer(cors)
        .into_inner();

    let mut server = Server::builder().accept_http1(true).layer(layer);

    let repository_index = config.database.instantiate_repository_index().await?;
    let service = SourceControlServer::new(Service::new(repository_index));
    let server = server.add_service(tonic_web::enable(service));

    info!("Listening on {}", args.listen_endpoint);

    server.serve(args.listen_endpoint).await.map_err(Into::into)
}
