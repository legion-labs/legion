//! Legion Source Control Server
//!
//! TODO: write documentation.

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use clap::Parser;
use lgn_source_control::{blob_storage::BlobStorageUrl, Commit, RepositoryQuery, RepositoryUrl};
use lgn_source_control_proto::source_control_server::{SourceControl, SourceControlServer};
use lgn_source_control_proto::{
    ClearLockRequest, ClearLockResponse, CommitExistsRequest, CommitExistsResponse,
    CommitToBranchRequest, CommitToBranchResponse, CountLocksInDomainRequest,
    CountLocksInDomainResponse, CreateRepositoryRequest, CreateRepositoryResponse,
    DestroyRepositoryRequest, DestroyRepositoryResponse, FindBranchRequest, FindBranchResponse,
    FindBranchesInLockDomainRequest, FindBranchesInLockDomainResponse, FindLockRequest,
    FindLockResponse, FindLocksInDomainRequest, FindLocksInDomainResponse,
    GetBlobStorageUrlRequest, GetBlobStorageUrlResponse, InsertBranchRequest, InsertBranchResponse,
    InsertCommitRequest, InsertCommitResponse, InsertLockRequest, InsertLockResponse,
    ReadBranchesRequest, ReadBranchesResponse, ReadCommitRequest, ReadCommitResponse,
    ReadTreeRequest, ReadTreeResponse, RegisterWorkspaceRequest, RegisterWorkspaceResponse,
    SaveTreeRequest, SaveTreeResponse, UpdateBranchRequest, UpdateBranchResponse,
};
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::{debug, info, LevelFilter};

struct Service {
    repository_queries: RwLock<HashMap<String, Arc<Box<dyn RepositoryQuery>>>>,
    sql_url: String,
    blob_storage_url: BlobStorageUrl,
}

impl Service {
    pub fn new(
        database_host: &str,
        database_username: Option<&str>,
        database_password: Option<&str>,
        blob_storage_url: BlobStorageUrl,
    ) -> Self {
        let sql_url = format!(
            "mysql://{}:{}@{}/",
            database_username.unwrap_or_default(),
            database_password.unwrap_or_default(),
            database_host,
        );

        Self {
            repository_queries: RwLock::new(HashMap::new()),
            sql_url,
            blob_storage_url,
        }
    }

    fn get_repository_url(&self, name: &str) -> RepositoryUrl {
        (self.sql_url.clone() + name).parse().unwrap()
    }

    fn get_repository_query(
        &self,
        name: &str,
    ) -> Result<Arc<Box<dyn RepositoryQuery>>, tonic::Status> {
        self.repository_queries
            .read()
            .unwrap()
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("repository not found"))
            .map(Arc::clone)
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
    async fn ping(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Ok(tonic::Response::new(()))
    }

    async fn create_repository(
        &self,
        request: tonic::Request<CreateRepositoryRequest>,
    ) -> Result<tonic::Response<CreateRepositoryResponse>, tonic::Status> {
        let origin = Self::get_request_origin(&request);
        let name = request.into_inner().repository_name;

        debug!("{}: Creating repository `{}`...", origin, &name);

        let repository_url = self.get_repository_url(&name);
        let repository_query = repository_url.into_query();

        let blob_storage_url = repository_query
            .create_repository(Some(self.blob_storage_url.clone()))
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        info!(
            "{}: Created repository `{}` with blob storage URL: {}",
            origin, &name, &blob_storage_url
        );

        self.repository_queries
            .write()
            .unwrap()
            .insert(name, Arc::new(repository_query));

        Ok(tonic::Response::new(CreateRepositoryResponse {
            blob_storage_url: blob_storage_url.to_string(),
        }))
    }

    async fn destroy_repository(
        &self,
        request: tonic::Request<DestroyRepositoryRequest>,
    ) -> Result<tonic::Response<DestroyRepositoryResponse>, tonic::Status> {
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
        let repository_query = self.repository_queries.write().unwrap().remove(&name);

        let repository_query = match repository_query {
            Some(repository_query) => repository_query,
            None => self.get_repository_query(&name)?,
        };

        repository_query
            .destroy_repository()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(DestroyRepositoryResponse {}))
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
        let query = self.get_repository_query(&request.repository_name)?;

        query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let branch = query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let branches = query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let branches = query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let commit = Some(
            query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let tree = Some(
            query
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
        let query = self.get_repository_query(&request.repository_name)?;

        query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let lock = query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let locks = query
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
        let query = self.get_repository_query(&request.repository_name)?;

        query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let commit: Result<Commit> = request.commit.unwrap_or_default().try_into();
        let commit = commit.map_err(|e| tonic::Status::unknown(e.to_string()))?;

        query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let commit: Result<Commit> = request.commit.unwrap_or_default().try_into();
        let commit = commit.map_err(|e| tonic::Status::unknown(e.to_string()))?;

        query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let exists = query
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
        let query = self.get_repository_query(&request.repository_name)?;

        query
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
        let query = self.get_repository_query(&request.repository_name)?;

        query
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
        let query = self.get_repository_query(&request.repository_name)?;

        query
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
        let query = self.get_repository_query(&request.repository_name)?;

        let count = query
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
        &args.database_host,
        args.database_username.as_deref(),
        args.database_password.as_deref(),
        args.blob_storage_url,
    ));
    let server = tonic::transport::Server::builder()
        .accept_http1(true)
        .add_service(service);

    server.serve(args.listen_endpoint).await.map_err(Into::into)
}
