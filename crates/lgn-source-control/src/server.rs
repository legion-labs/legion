//! Legion Source Control Server
//!
//! TODO: write documentation.

// crate-specific lint exceptions:
//#![allow()]

use std::{net::SocketAddr, sync::Arc};

use crate::{
    api::source_control::{
        server::{
            CommitToBranchRequest, CommitToBranchResponse, CountLocksRequest, CountLocksResponse,
            CreateBranchRequest, CreateBranchResponse, CreateRepositoryRequest,
            CreateRepositoryResponse, DeleteRepositoryRequest, DeleteRepositoryResponse,
            GetBranchRequest, GetBranchResponse, GetLockRequest, GetLockResponse,
            ListBranchesRequest, ListBranchesResponse, ListCommitsRequest, ListCommitsResponse,
            ListLocksRequest, ListLocksResponse, ListRepositoriesRequest, ListRepositoriesResponse,
            LockRequest, LockResponse, RepositoryExistsRequest, RepositoryExistsResponse,
            UnlockRequest, UnlockResponse, UpdateBranchRequest, UpdateBranchResponse,
        },
        Api,
    },
    Index, ListBranchesQuery, ListCommitsQuery, ListLocksQuery, RepositoryIndex, RepositoryName,
    SqlRepositoryIndex,
};
use async_trait::async_trait;
use axum::extract::ConnectInfo;
use lgn_online::server::{Error, Result};
use lgn_tracing::{debug, info, warn};

pub struct Server {
    repository_index: SqlRepositoryIndex,
}

impl Server {
    pub fn new(repository_index: SqlRepositoryIndex) -> Self {
        Self { repository_index }
    }

    async fn get_index_for_repository(
        &self,
        repository_name: &RepositoryName,
    ) -> Result<Box<dyn Index>> {
        self.repository_index
            .load_repository(repository_name)
            .await
            .map_err(|e| Error::internal(e.to_string()))
    }

    /// This requires the usage of `Router::into_make_service_with_connect_info` to run your server
    /// otherwise it will be unable to retrieve the `ConnectInfo` and will return the default value.
    fn get_request_origin(parts: &http::request::Parts) -> String {
        parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map_or_else(|| "unknown".to_string(), |c| c.0.to_string())
    }
}

#[async_trait]
impl Api for Arc<Server> {
    async fn create_repository(
        &self,
        request: CreateRepositoryRequest,
    ) -> Result<CreateRepositoryResponse> {
        let origin = Server::get_request_origin(&request.parts);
        let repository_name = request
            .body
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        debug!("{}: Creating repository `{}`...", origin, &repository_name);

        match self
            .repository_index
            .create_repository(&repository_name)
            .await
        {
            Ok(blob_storage_url) => blob_storage_url,
            Err(crate::Error::RepositoryAlreadyExists { repository_name }) => {
                warn!(
                    "{}: Did not create repository `{}` as it already exists",
                    origin, repository_name
                );

                return Ok(CreateRepositoryResponse::Status409 {});
            }
            Err(e) => return Err(Error::internal(e.to_string())),
        };

        info!("{}: Created repository.", origin);

        Ok(CreateRepositoryResponse::Status201 {})
    }

    async fn delete_repository(
        &self,
        request: DeleteRepositoryRequest,
    ) -> Result<DeleteRepositoryResponse> {
        let origin = Server::get_request_origin(&request.parts);
        let repository_name: RepositoryName = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        match self
            .repository_index
            .destroy_repository(&repository_name)
            .await
        {
            Ok(()) => {
                info!("{}: Destroyed repository `{}`", origin, &repository_name);

                Ok(DeleteRepositoryResponse::Status204 {})
            }
            Err(crate::Error::RepositoryNotFound { repository_name: _ }) => {
                warn!(
                    "{}: Did not destroy repository `{}` as it does not exist",
                    origin, &repository_name
                );
                Ok(DeleteRepositoryResponse::Status404 {})
            }
            Err(e) => Err(Error::internal(e.to_string())),
        }
    }

    async fn repository_exists(
        &self,
        request: RepositoryExistsRequest,
    ) -> Result<RepositoryExistsResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        match self
            .repository_index
            .load_repository(&repository_name)
            .await
        {
            Ok(_) => Ok(RepositoryExistsResponse::Status200 {}),
            Err(crate::Error::RepositoryNotFound { repository_name: _ }) => {
                Ok(RepositoryExistsResponse::Status404 {})
            }
            Err(e) => Err(Error::internal(e.to_string())),
        }
    }

    async fn list_repositories(
        &self,
        _request: ListRepositoriesRequest,
    ) -> Result<ListRepositoriesResponse> {
        let repository_names = self
            .repository_index
            .list_repositories()
            .await
            .map_err(|err| Error::internal(err.to_string()))?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<crate::api::source_control::RepositoryName>>();

        Ok(ListRepositoriesResponse::Status200(repository_names.into()))
    }

    async fn get_branch(&self, request: GetBranchRequest) -> Result<GetBranchResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let branch_name = request
            .branch_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let index = self.get_index_for_repository(&repository_name).await?;

        match index.get_branch(&branch_name).await {
            Ok(branch) => Ok(GetBranchResponse::Status200(branch.into())),
            Err(crate::Error::BranchNotFound { .. }) => Ok(GetBranchResponse::Status404),
            Err(err) => return Err(Error::internal(err.to_string())),
        }
    }

    async fn list_branches(&self, request: ListBranchesRequest) -> Result<ListBranchesResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let index = self.get_index_for_repository(&repository_name).await?;

        let lock_domain_id = request.lock_domain_id.map(|id| id.0);

        let query = ListBranchesQuery {
            lock_domain_id: lock_domain_id.as_deref(),
        };

        let branches = index
            .list_branches(&query)
            .await
            .map_err(|e| Error::internal(e.to_string()))?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<crate::api::source_control::Branch>>();

        Ok(ListBranchesResponse::Status200(branches.into()))
    }

    async fn list_commits(&self, request: ListCommitsRequest) -> Result<ListCommitsResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let branch_name = request
            .branch_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let index = self.get_index_for_repository(&repository_name).await?;

        let query = ListCommitsQuery {
            branch_name,
            commit_ids: request.commit_ids.into_iter().map(Into::into).collect(),
            depth: request.depth,
        };

        let commits = index
            .list_commits(&query)
            .await
            .map_err(|e| Error::internal(e.to_string()))?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<crate::api::source_control::Commit>>();

        Ok(ListCommitsResponse::Status200(commits.into()))
    }

    async fn lock(&self, request: LockRequest) -> Result<LockResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;
        let index = self.get_index_for_repository(&repository_name).await?;

        let lock = request
            .body
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        index
            .lock(&lock)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        Ok(LockResponse::Status201)
    }

    async fn get_lock(&self, request: GetLockRequest) -> Result<GetLockResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;
        let index = self.get_index_for_repository(&repository_name).await?;

        match index
            .get_lock(
                &request.lock_domain_id.0,
                &request
                    .canonical_path
                    .try_into()
                    .map_err(|e| Error::bad_request(format!("{}", e)))?,
            )
            .await
        {
            Ok(lock) => Ok(GetLockResponse::Status200(lock.into())),
            Err(crate::Error::LockNotFound { .. }) => Ok(GetLockResponse::Status404),
            Err(err) => return Err(Error::internal(err.to_string())),
        }
    }

    async fn list_locks(&self, request: ListLocksRequest) -> Result<ListLocksResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let index = self.get_index_for_repository(&repository_name).await?;

        let lock_domain_ids = request
            .lock_domain_ids
            .into_iter()
            .map(|id| id.0)
            .collect::<Vec<String>>();

        let query = ListLocksQuery {
            lock_domain_ids: lock_domain_ids.iter().map(String::as_str).collect(),
        };

        let locks = index
            .list_locks(&query)
            .await
            .map_err(|e| Error::internal(e.to_string()))?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<crate::api::source_control::Lock>>();

        Ok(ListLocksResponse::Status200(locks.into()))
    }

    async fn commit_to_branch(
        &self,
        request: CommitToBranchRequest,
    ) -> Result<CommitToBranchResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let index = self.get_index_for_repository(&repository_name).await?;

        let branch_name = request
            .branch_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let new_commit = request
            .body
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let commit = index
            .commit_to_branch(&branch_name, new_commit)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        Ok(CommitToBranchResponse::Status201(commit.into()))
    }

    async fn update_branch(&self, request: UpdateBranchRequest) -> Result<UpdateBranchResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let branch_name = request
            .branch_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let index = self.get_index_for_repository(&repository_name).await?;

        let branch = index
            .update_branch(&branch_name, request.body.into())
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        Ok(UpdateBranchResponse::Status200(branch.into()))
    }

    async fn create_branch(&self, request: CreateBranchRequest) -> Result<CreateBranchResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let index = self.get_index_for_repository(&repository_name).await?;

        let new_branch = request
            .body
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;

        let branch = index
            .insert_branch(new_branch)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        Ok(CreateBranchResponse::Status201(branch.into()))
    }

    async fn unlock(&self, request: UnlockRequest) -> Result<UnlockResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;
        let index = self.get_index_for_repository(&repository_name).await?;

        index
            .unlock(
                &request.lock_domain_id.0,
                &request
                    .canonical_path
                    .try_into()
                    .map_err(|e| Error::bad_request(format!("{}", e)))?,
            )
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        Ok(UnlockResponse::Status204)
    }

    async fn count_locks(&self, request: CountLocksRequest) -> Result<CountLocksResponse> {
        let repository_name = request
            .repository_name
            .try_into()
            .map_err(|e| Error::bad_request(format!("{}", e)))?;
        let index = self.get_index_for_repository(&repository_name).await?;

        let lock_domain_ids = request
            .lock_domain_ids
            .into_iter()
            .map(|id| id.0)
            .collect::<Vec<String>>();

        let query = ListLocksQuery {
            lock_domain_ids: lock_domain_ids.iter().map(String::as_str).collect(),
        };

        let count = index
            .count_locks(&query)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        Ok(CountLocksResponse::Status204 {
            x_locks_count: count,
        })
    }
}
