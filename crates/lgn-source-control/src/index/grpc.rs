use std::sync::Arc;

use async_trait::async_trait;
use lgn_tracing::prelude::*;
use tokio::sync::Mutex;
use tonic::codegen::{Body, StdError};

use lgn_source_control_proto::{
    source_control_client::SourceControlClient, CommitToBranchRequest, CountLocksRequest,
    CreateRepositoryRequest, DestroyRepositoryRequest, GetBranchRequest, GetLockRequest,
    GetTreeRequest, InsertBranchRequest, ListBranchesRequest, ListCommitsRequest, ListLocksRequest,
    LockRequest, RegisterWorkspaceRequest, RepositoryExistsRequest, SaveTreeRequest, UnlockRequest,
    UpdateBranchRequest,
};

use crate::{
    Branch, CanonicalPath, Commit, CommitId, Error, Index, ListBranchesQuery, ListCommitsQuery,
    ListLocksQuery, Lock, MapOtherError, RepositoryIndex, RepositoryName, Result, Tree,
    WorkspaceRegistration,
};

// Access to a source-control repository index through a gRPC server.
pub struct GrpcRepositoryIndex<C> {
    client: Arc<Mutex<SourceControlClient<C>>>,
}

impl<C> GrpcRepositoryIndex<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(client: C) -> Self {
        Self {
            client: Arc::new(Mutex::new(SourceControlClient::new(client))),
        }
    }
}

#[async_trait]
impl<C> RepositoryIndex for GrpcRepositoryIndex<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + 'static,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn create_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("GrpcRepositoryIndex::create_repository");

        let resp = self
            .client
            .lock()
            .await
            .create_repository(CreateRepositoryRequest {
                repository_name: repository_name.to_string(),
            })
            .await
            .map_other_err(format!(
                "failed to create repository `{}`",
                &repository_name
            ))?
            .into_inner();

        if resp.already_exists {
            return Err(Error::repository_already_exists(repository_name));
        }

        Ok(Box::new(GrpcIndex::new(
            repository_name,
            self.client.clone(),
        )))
    }

    async fn destroy_repository(&self, repository_name: RepositoryName) -> Result<()> {
        async_span_scope!("GrpcRepositoryIndex::destroy_repository");

        let resp = self
            .client
            .lock()
            .await
            .destroy_repository(DestroyRepositoryRequest {
                repository_name: repository_name.to_string(),
            })
            .await
            .map_other_err(format!(
                "failed to destroy repository `{}`",
                repository_name
            ))?
            .into_inner();

        if resp.does_not_exist {
            return Err(Error::repository_does_not_exist(repository_name));
        }

        Ok(())
    }

    async fn load_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("GrpcRepositoryIndex::load_repository");

        let resp = self
            .client
            .lock()
            .await
            .repository_exists(RepositoryExistsRequest {
                repository_name: repository_name.to_string(),
            })
            .await
            .map_other_err(format!("failed to open repository `{}`", repository_name))?
            .into_inner();

        if !resp.exists {
            return Err(Error::repository_does_not_exist(repository_name.to_owned()));
        }

        Ok(Box::new(GrpcIndex::new(
            repository_name.to_owned(),
            self.client.clone(),
        )))
    }
}

// Access to a source-control repository through a gRPC server.
pub struct GrpcIndex<C> {
    repository_name: RepositoryName,
    client: Arc<Mutex<SourceControlClient<C>>>,
}

impl<C> GrpcIndex<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    fn new(repository_name: RepositoryName, client: Arc<Mutex<SourceControlClient<C>>>) -> Self {
        Self {
            repository_name,
            client,
        }
    }
}

#[async_trait]
impl<C> Index for GrpcIndex<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    fn repository_name(&self) -> &RepositoryName {
        &self.repository_name
    }

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        async_span_scope!("register_workspace");
        self.client
            .lock()
            .await
            .register_workspace(RegisterWorkspaceRequest {
                repository_name: self.repository_name.to_string(),
                workspace_registration: Some(workspace_registration.clone().into()),
            })
            .await
            .map_other_err(format!(
                "failed to register workspace `{}`",
                workspace_registration.id
            ))
            .map(|_| ())
    }

    async fn get_branch(&self, branch_name: &str) -> Result<Branch> {
        async_span_scope!("GrpcIndexBackend::get_branch");
        let resp = self
            .client
            .lock()
            .await
            .get_branch(GetBranchRequest {
                repository_name: self.repository_name.to_string(),
                branch_name: branch_name.into(),
            })
            .await
            .map_other_err(format!("failed to find branch `{}`", branch_name))?
            .into_inner();

        match resp.branch {
            Some(branch) => Ok(branch.into()),
            None => Err(Error::branch_not_found(branch_name.to_string())),
        }
    }

    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>> {
        async_span_scope!("GrpcIndexBackend::list_branches");
        let resp = self
            .client
            .lock()
            .await
            .list_branches(ListBranchesRequest {
                repository_name: self.repository_name.to_string(),
                lock_domain_id: query.lock_domain_id.unwrap_or_default().into(),
            })
            .await
            .map_other_err("failed to read branches")?
            .into_inner();

        Ok(resp.branches.into_iter().map(Into::into).collect())
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        async_span_scope!("GrpcIndexBackend::insert_branch");
        self.client
            .lock()
            .await
            .insert_branch(InsertBranchRequest {
                repository_name: self.repository_name.to_string(),
                branch: Some(branch.clone().into()),
            })
            .await
            .map_other_err(format!("failed to insert branch `{}`", branch.name))
            .map(|_| ())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        async_span_scope!("GrpcIndexBackend::update_branch");
        self.client
            .lock()
            .await
            .update_branch(UpdateBranchRequest {
                repository_name: self.repository_name.to_string(),
                branch: Some(branch.clone().into()),
            })
            .await
            .map_other_err(format!("failed to update branch `{}`", branch.name))
            .map(|_| ())
    }

    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        async_span_scope!("GrpcIndexBackend::list_commits");
        let resp = self
            .client
            .lock()
            .await
            .list_commits(ListCommitsRequest {
                repository_name: self.repository_name.to_string(),
                commit_ids: query
                    .commit_ids
                    .iter()
                    .map(|commit_id| commit_id.0)
                    .collect(),
                depth: query.depth,
            })
            .await
            .map_other_err("failed to list commits")?
            .into_inner();

        resp.commits
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()
            .map_other_err("failed to parse commits")
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<CommitId> {
        async_span_scope!("GrpcIndexBackend::commit_to_branch");
        let resp = self
            .client
            .lock()
            .await
            .commit_to_branch(CommitToBranchRequest {
                repository_name: self.repository_name.to_string(),
                commit: Some(commit.clone().into()),
                branch: Some(branch.clone().into()),
            })
            .await
            .map_other_err(format!(
                "failed to commit `{}` to branch `{}`",
                commit.id, branch.name
            ))?
            .into_inner();

        Ok(CommitId(resp.commit_id))
    }

    async fn get_tree(&self, id: &str) -> Result<Tree> {
        async_span_scope!("GrpcIndexBackend::get_tree");
        let resp = self
            .client
            .lock()
            .await
            .get_tree(GetTreeRequest {
                repository_name: self.repository_name.to_string(),
                tree_id: id.into(),
            })
            .await
            .map_other_err(format!("failed to get tree `{}`", id))?
            .into_inner();

        resp.tree.unwrap_or_default().try_into()
    }

    async fn save_tree(&self, tree: &Tree) -> Result<String> {
        async_span_scope!("GrpcIndexBackend::save_tree");
        self.client
            .lock()
            .await
            .save_tree(SaveTreeRequest {
                repository_name: self.repository_name.to_string(),
                tree: Some(tree.clone().into()),
            })
            .await
            .map_other_err("failed to save tree")
            .map(|resp| resp.into_inner().tree_id)
    }

    async fn lock(&self, lock: &Lock) -> Result<()> {
        async_span_scope!("GrpcIndexBackend::lock");
        self.client
            .lock()
            .await
            .lock(LockRequest {
                repository_name: self.repository_name.to_string(),
                lock: Some(lock.clone().into()),
            })
            .await
            .map_other_err(format!(
                "failed to create lock `{}/{}`",
                lock.lock_domain_id, lock.canonical_path,
            ))
            .map(|_| ())
    }

    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()> {
        async_span_scope!("GrpcIndexBackend::unlock");
        self.client
            .lock()
            .await
            .unlock(UnlockRequest {
                repository_name: self.repository_name.to_string(),
                lock_domain_id: lock_domain_id.into(),
                canonical_path: canonical_path.to_string(),
            })
            .await
            .map_other_err(format!(
                "failed to clear lock `{}/{}`",
                lock_domain_id, canonical_path,
            ))
            .map(|_| ())
    }

    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock> {
        async_span_scope!("GrpcIndexBackend::get_lock");
        let resp = self
            .client
            .lock()
            .await
            .get_lock(GetLockRequest {
                repository_name: self.repository_name.to_string(),
                lock_domain_id: lock_domain_id.into(),
                canonical_path: canonical_path.to_string(),
            })
            .await
            .map_other_err(format!(
                "failed to find lock `{}` in lock domain `{}`",
                canonical_path, lock_domain_id
            ))?
            .into_inner();

        match resp.lock {
            Some(lock) => Ok(lock.try_into()?),
            None => Err(Error::lock_not_found(
                lock_domain_id.to_string(),
                canonical_path.clone(),
            )),
        }
    }

    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>> {
        async_span_scope!("GrpcIndexBackend::list_locks");
        let resp = self
            .client
            .lock()
            .await
            .list_locks(ListLocksRequest {
                repository_name: self.repository_name.to_string(),
                lock_domain_ids: query
                    .lock_domain_ids
                    .iter()
                    .copied()
                    .map(Into::into)
                    .collect(),
            })
            .await
            .map_other_err("failed to list locks")?
            .into_inner();

        resp.locks.into_iter().map(TryInto::try_into).collect()
    }

    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32> {
        async_span_scope!("GrpcIndexBackend::count_locks");
        let resp = self
            .client
            .lock()
            .await
            .count_locks(CountLocksRequest {
                repository_name: self.repository_name.to_string(),
                lock_domain_ids: query
                    .lock_domain_ids
                    .iter()
                    .copied()
                    .map(Into::into)
                    .collect(),
            })
            .await
            .map_other_err("failed to count locks")?
            .into_inner();

        Ok(resp.count)
    }
}
