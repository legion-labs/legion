use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use http::Uri;
use lgn_governance::types::SpaceId;
use lgn_tracing::prelude::*;

use crate::{
    api::source_control::{
        client::Client,
        client::{
            CommitToBranchRequest, CommitToBranchResponse, CountLocksRequest, CountLocksResponse,
            CreateBranchRequest, CreateBranchResponse, CreateRepositoryRequest,
            CreateRepositoryResponse, DeleteRepositoryRequest, DeleteRepositoryResponse,
            GetBranchRequest, GetBranchResponse, GetLockRequest, GetLockResponse,
            ListBranchesRequest, ListBranchesResponse, ListCommitsRequest, ListCommitsResponse,
            ListLocksRequest, ListLocksResponse, ListRepositoriesRequest, ListRepositoriesResponse,
            LockRequest, LockResponse, RepositoryExistsRequest, RepositoryExistsResponse,
            UnlockRequest, UnlockResponse, UpdateBranchRequest, UpdateBranchResponse,
        },
    },
    BranchName, NewBranch, NewCommit, UpdateBranch,
};

use crate::{
    Branch, CanonicalPath, Commit, Error, Index, ListBranchesQuery, ListCommitsQuery,
    ListLocksQuery, Lock, RepositoryIndex, RepositoryName, Result,
};

// Access to a source-control repository index through an `HTTP` Api.
pub struct ApiRepositoryIndex<C> {
    client: Arc<Client<C>>,
    space_id: SpaceId,
}

impl<C, ResBody> ApiRepositoryIndex<C>
where
    C: tower::Service<http::Request<hyper::Body>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + Sync
        + Debug
        + 'static,
    C::Error: Into<lgn_online::client::Error>,
    C::Future: Send,
    ResBody: hyper::body::HttpBody + Send,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error,
{
    pub fn new(inner: C, base_url: Uri, space_id: SpaceId) -> Self {
        Self {
            client: Arc::new(Client::new(inner, base_url)),
            space_id,
        }
    }
}

#[async_trait]
impl<C, ResBody> RepositoryIndex for ApiRepositoryIndex<C>
where
    C: tower::Service<http::Request<hyper::Body>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + Sync
        + Debug
        + 'static,
    C::Error: Into<lgn_online::client::Error>,
    C::Future: Send,
    ResBody: hyper::body::HttpBody + Send,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error,
{
    async fn create_repository(&self, repository_name: &RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("ApiRepositoryIndex::create_repository");

        let resp = self
            .client
            .create_repository(CreateRepositoryRequest {
                space_id: self.space_id.clone().into(),
                body: repository_name.clone().into(),
            })
            .await
            .map_err(|e| {
                anyhow::anyhow!("failed to create repository `{}`: {}", repository_name, e)
            })?;

        match resp {
            CreateRepositoryResponse::Status201 { .. } => Ok(Box::new(ApiIndex::new(
                self.client.clone(),
                repository_name.clone(),
                self.space_id.clone(),
            ))),
            CreateRepositoryResponse::Status409 { .. } => {
                return Err(Error::repository_already_exists(repository_name.clone()));
            }
        }
    }

    async fn destroy_repository(&self, repository_name: &RepositoryName) -> Result<()> {
        async_span_scope!("ApiRepositoryIndex::destroy_repository");

        let resp = self
            .client
            .delete_repository(DeleteRepositoryRequest {
                space_id: self.space_id.clone().into(),
                repository_name: repository_name.clone().into(),
            })
            .await
            .map_err(|e| {
                anyhow::anyhow!("failed to destroy repository `{}`: {}", repository_name, e)
            })?;

        match resp {
            DeleteRepositoryResponse::Status204 { .. } => Ok(()),
            DeleteRepositoryResponse::Status404 { .. } => {
                return Err(Error::repository_not_found(repository_name.clone()));
            }
        }
    }

    async fn load_repository(&self, repository_name: &RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("ApiRepositoryIndex::load_repository");

        let resp = self
            .client
            .repository_exists(RepositoryExistsRequest {
                space_id: self.space_id.clone().into(),
                repository_name: repository_name.clone().into(),
            })
            .await
            .map_err(|e| {
                anyhow::anyhow!("failed to open repository `{}`: {}", repository_name, e)
            })?;

        match resp {
            RepositoryExistsResponse::Status200 { .. } => Ok(Box::new(ApiIndex::new(
                self.client.clone(),
                repository_name.clone(),
                self.space_id.clone(),
            ))),
            RepositoryExistsResponse::Status404 { .. } => {
                return Err(Error::repository_not_found(repository_name.clone()));
            }
        }
    }

    async fn list_repositories(&self) -> Result<Vec<RepositoryName>> {
        async_span_scope!("ApiRepositoryIndex::list_repositories");

        let resp = self
            .client
            .list_repositories(ListRepositoriesRequest {
                space_id: self.space_id.clone().into(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("failed to list repositories: {}", e))?;

        match resp {
            ListRepositoriesResponse::Status200 { body, .. } => Ok(body
                .0
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<RepositoryName>>>()
                .map_err(|e| anyhow::anyhow!("failed to parse repositories: {}", e))?),
        }
    }
}

// Access to a source-control repository through a `HTTP` Api.
pub struct ApiIndex<C> {
    client: Arc<Client<C>>,
    repository_name: RepositoryName,
    space_id: SpaceId,
}

impl<C, ResBody> ApiIndex<C>
where
    C: tower::Service<http::Request<hyper::Body>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + Sync
        + Debug
        + 'static,
    C::Error: Into<lgn_online::client::Error>,
    C::Future: Send,
    ResBody: hyper::body::HttpBody + Send,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error,
{
    fn new(client: Arc<Client<C>>, repository_name: RepositoryName, space_id: SpaceId) -> Self {
        Self {
            client,
            repository_name,
            space_id,
        }
    }
}

#[async_trait]
impl<C, ResBody> Index for ApiIndex<C>
where
    C: tower::Service<http::Request<hyper::Body>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + Sync
        + Debug
        + 'static,
    C::Error: Into<lgn_online::client::Error>,
    C::Future: Send,
    ResBody: hyper::body::HttpBody + Send,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error,
{
    fn repository_name(&self) -> &RepositoryName {
        &self.repository_name
    }

    async fn get_branch(&self, branch_name: &BranchName) -> Result<Branch> {
        async_span_scope!("ApiIndex::get_branch");
        let resp = self
            .client
            .get_branch(GetBranchRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                branch_name: branch_name.clone().into(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("failed to find branch `{}`: {}", branch_name, e))?;

        match resp {
            GetBranchResponse::Status200 { body, .. } => Ok(body
                .try_into()
                .map_err(|e| anyhow::anyhow!("failed to parse branch: {}", e))?),
            GetBranchResponse::Status404 { .. } => {
                return Err(Error::branch_not_found(branch_name.clone()));
            }
        }
    }

    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>> {
        async_span_scope!("ApiIndex::list_branches");
        let resp = self
            .client
            .list_branches(ListBranchesRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                lock_domain_id: query
                    .lock_domain_id
                    .map(ToString::to_string)
                    .map(Into::into),
            })
            .await
            .map_err(|e| anyhow::anyhow!("failed to list branches: {}", e))?;

        match resp {
            ListBranchesResponse::Status200 { body, .. } => Ok(body
                .0
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Branch>>>()
                .map_err(|e| anyhow::anyhow!("failed to parse branches: {}", e))?),
        }
    }

    async fn insert_branch(&self, new_branch: NewBranch) -> Result<Branch> {
        async_span_scope!("ApiIndex::insert_branch");
        let resp = self
            .client
            .create_branch(CreateBranchRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                body: new_branch.clone().into(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("failed to insert branch `{}`: {}", new_branch.name, e))?;

        match resp {
            CreateBranchResponse::Status201 { body, .. } => Ok(body
                .try_into()
                .map_err(|e| anyhow::anyhow!("failed to parse branch: {}", e))?),
            CreateBranchResponse::Status409 { .. } => {
                return Err(Error::branch_already_exists(new_branch.name.clone()));
            }
        }
    }

    async fn update_branch(
        &self,
        branch_name: &BranchName,
        update_branch: UpdateBranch,
    ) -> Result<Branch> {
        async_span_scope!("ApiIndex::update_branch");
        let resp = self
            .client
            .update_branch(UpdateBranchRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                branch_name: branch_name.clone().into(),
                body: update_branch.into(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("failed to update branch `{}`: {}", branch_name, e))?;

        match resp {
            UpdateBranchResponse::Status200 { body, .. } => Ok(body
                .try_into()
                .map_err(|e| anyhow::anyhow!("failed to parse branch: {}", e))?),
            UpdateBranchResponse::Status404 { .. } => {
                return Err(Error::branch_not_found(branch_name.clone()));
            }
        }
    }

    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        async_span_scope!("ApiIndex::list_commits");
        let resp = self
            .client
            .list_commits(ListCommitsRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                branch_name: query.branch_name.clone().into(),
                commit_ids: query.commit_ids.iter().copied().map(Into::into).collect(),
                depth: query.depth,
            })
            .await
            .map_err(|e| anyhow::anyhow!("failed to list commits: {}", e))?;

        match resp {
            ListCommitsResponse::Status200 { body, .. } => Ok(body
                .0
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Commit>>>()
                .map_err(|e| anyhow::anyhow!("failed to parse commits: {}", e))?),
        }
    }

    async fn commit_to_branch(
        &self,
        branch_name: &BranchName,
        new_commit: NewCommit,
    ) -> Result<Commit> {
        async_span_scope!("ApiIndex::commit_to_branch");
        let resp = self
            .client
            .commit_to_branch(CommitToBranchRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                branch_name: branch_name.clone().into(),
                body: new_commit.clone().into(),
            })
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "failed to commit `{:?}` to branch `{}`: {}",
                    new_commit,
                    branch_name,
                    e
                )
            })?;

        match resp {
            CommitToBranchResponse::Status201 { body, .. } => Ok(body
                .try_into()
                .map_err(|e| anyhow::anyhow!("failed to parse commit: {}", e))?),
        }
    }

    async fn lock(&self, lock: &Lock) -> Result<()> {
        async_span_scope!("ApiIndex::lock");
        let resp = self
            .client
            .lock(LockRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                body: lock.clone().into(),
            })
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "failed to create lock `{}/{}`: {}",
                    lock.lock_domain_id,
                    lock.canonical_path,
                    e
                )
            })?;

        match resp {
            LockResponse::Status201 { .. } => Ok(()),
            LockResponse::Status409 { .. } => {
                return Err(Error::lock_already_exists(lock.clone()));
            }
        }
    }

    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()> {
        async_span_scope!("ApiIndex::unlock");
        let resp = self
            .client
            .unlock(UnlockRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                lock_domain_id: lock_domain_id.to_string().into(),
                canonical_path: canonical_path.to_string().into(),
            })
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "failed to clear lock `{}/{}`: {}",
                    lock_domain_id,
                    canonical_path,
                    e
                )
            })?;

        match resp {
            UnlockResponse::Status204 { .. } => Ok(()),
            UnlockResponse::Status404 { .. } => {
                return Err(Error::lock_not_found(
                    lock_domain_id.to_string(),
                    canonical_path.clone(),
                ));
            }
        }
    }

    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock> {
        async_span_scope!("ApiIndex::get_lock");
        let resp = self
            .client
            .get_lock(GetLockRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                lock_domain_id: lock_domain_id.to_string().into(),
                canonical_path: canonical_path.to_string().into(),
            })
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "failed to find lock `{}/{}`: {}",
                    lock_domain_id,
                    canonical_path,
                    e
                )
            })?;

        match resp {
            GetLockResponse::Status200 { body, .. } => Ok(body
                .try_into()
                .map_err(|e| anyhow::anyhow!("failed to parse lock: {}", e))?),
            GetLockResponse::Status404 { .. } => {
                return Err(Error::lock_not_found(
                    lock_domain_id.to_string(),
                    canonical_path.clone(),
                ));
            }
        }
    }

    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>> {
        async_span_scope!("ApiIndex::list_locks");
        let resp = self
            .client
            .list_locks(ListLocksRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                lock_domain_ids: query
                    .lock_domain_ids
                    .iter()
                    .map(ToString::to_string)
                    .map(Into::into)
                    .collect(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("failed to list locks: {}", e))?;

        match resp {
            ListLocksResponse::Status200 { body, .. } => Ok(body
                .0
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Lock>>>()
                .map_err(|e| anyhow::anyhow!("failed to parse locks: {}", e))?),
        }
    }

    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32> {
        async_span_scope!("ApiIndex::count_locks");
        let resp = self
            .client
            .count_locks(CountLocksRequest {
                space_id: self.space_id.clone().into(),
                repository_name: self.repository_name.clone().into(),
                lock_domain_ids: query
                    .lock_domain_ids
                    .iter()
                    .map(ToString::to_string)
                    .map(Into::into)
                    .collect(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("failed to count locks: {}", e))?;

        match resp {
            CountLocksResponse::Status204 { x_locks_count, .. } => Ok(x_locks_count),
        }
    }
}
