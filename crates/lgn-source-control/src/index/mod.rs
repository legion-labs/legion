mod grpc;
mod local;
mod sql;

use async_trait::async_trait;
use lgn_tracing::info;

use crate::{
    Branch, CanonicalPath, Commit, CommitId, Error, Lock, RepositoryName, Result, Tree,
    WorkspaceRegistration,
};

pub use grpc::*;
pub use local::*;
pub use sql::*;

/// Represents a source control index.
/// The query options for the `list_branches` method.
#[derive(Default, Clone, Debug)]
pub struct ListBranchesQuery<'q> {
    pub lock_domain_id: Option<&'q str>,
}

/// The query options for the `list_commits` method.
#[derive(Default, Clone, Debug)]
pub struct ListCommitsQuery {
    pub commit_ids: Vec<CommitId>,
    pub depth: u32,
}

impl ListCommitsQuery {
    pub fn single(commit_id: CommitId) -> Self {
        Self {
            commit_ids: vec![commit_id],
            ..Self::default()
        }
    }
}

/// The query options for the `list_locks` method.
#[derive(Default, Clone, Debug)]
pub struct ListLocksQuery<'q> {
    pub lock_domain_ids: Vec<&'q str>,
}

#[async_trait]
pub trait RepositoryIndex: Send + Sync {
    async fn create_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>>;
    async fn destroy_repository(&self, repository_name: RepositoryName) -> Result<()>;
    async fn load_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>>;
    async fn list_repositories(&self) -> Result<Vec<RepositoryName>>;

    async fn ensure_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        info!("Ensuring that repository `{}` exists...", repository_name);

        match self.load_repository(repository_name.clone()).await {
            Ok(index) => {
                info!(
                    "Repository `{}` exists already: loading it...",
                    repository_name
                );

                Ok(index)
            }
            Err(Error::RepositoryDoesNotExist { .. }) => {
                info!(
                    "Repository `{}` does not exist yet: creating it...",
                    repository_name
                );
                self.create_repository(repository_name).await
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
pub trait Index: Send + Sync {
    fn repository_name(&self) -> &RepositoryName;

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()>;

    async fn insert_branch(&self, branch: &Branch) -> Result<()>;
    async fn update_branch(&self, branch: &Branch) -> Result<()>;
    async fn get_branch(&self, branch_name: &str) -> Result<Branch>;
    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>>;

    async fn get_commit(&self, commit_id: CommitId) -> Result<Commit> {
        self.list_commits(&ListCommitsQuery {
            commit_ids: vec![commit_id],
            depth: 1,
        })
        .await?
        .pop()
        .ok_or_else(|| Error::commit_not_found(commit_id))
    }

    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>>;
    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<CommitId>;

    async fn get_tree(&self, id: &str) -> Result<Tree>;
    async fn save_tree(&self, tree: &Tree) -> Result<String>;

    async fn lock(&self, lock: &Lock) -> Result<()>;
    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()>;
    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock>;
    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>>;
    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32>;
}

// Blanket implementations.

#[async_trait]
impl<T: RepositoryIndex + ?Sized> RepositoryIndex for Box<T> {
    async fn create_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        self.as_ref().create_repository(repository_name).await
    }

    async fn destroy_repository(&self, repository_name: RepositoryName) -> Result<()> {
        self.as_ref().destroy_repository(repository_name).await
    }

    async fn load_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        self.as_ref().load_repository(repository_name).await
    }

    async fn list_repositories(&self) -> Result<Vec<RepositoryName>> {
        self.as_ref().list_repositories().await
    }
}

#[async_trait]
impl<T: RepositoryIndex> RepositoryIndex for &T {
    async fn create_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        (**self).create_repository(repository_name).await
    }

    async fn destroy_repository(&self, repository_name: RepositoryName) -> Result<()> {
        (**self).destroy_repository(repository_name).await
    }

    async fn load_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        (**self).load_repository(repository_name).await
    }

    async fn list_repositories(&self) -> Result<Vec<RepositoryName>> {
        (**self).list_repositories().await
    }
}

#[async_trait]
impl<T: Index + ?Sized> Index for Box<T> {
    fn repository_name(&self) -> &RepositoryName {
        self.as_ref().repository_name()
    }

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        self.as_ref()
            .register_workspace(workspace_registration)
            .await
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        self.as_ref().insert_branch(branch).await
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.as_ref().update_branch(branch).await
    }

    async fn get_branch(&self, branch_name: &str) -> Result<Branch> {
        self.as_ref().get_branch(branch_name).await
    }

    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>> {
        self.as_ref().list_branches(query).await
    }

    async fn get_commit(&self, commit_id: CommitId) -> Result<Commit> {
        self.as_ref().get_commit(commit_id).await
    }

    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        self.as_ref().list_commits(query).await
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<CommitId> {
        self.as_ref().commit_to_branch(commit, branch).await
    }

    async fn get_tree(&self, id: &str) -> Result<Tree> {
        self.as_ref().get_tree(id).await
    }

    async fn save_tree(&self, tree: &Tree) -> Result<String> {
        self.as_ref().save_tree(tree).await
    }

    async fn lock(&self, lock: &Lock) -> Result<()> {
        self.as_ref().lock(lock).await
    }

    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()> {
        self.as_ref().unlock(lock_domain_id, canonical_path).await
    }

    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock> {
        self.as_ref().get_lock(lock_domain_id, canonical_path).await
    }

    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>> {
        self.as_ref().list_locks(query).await
    }

    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32> {
        self.as_ref().count_locks(query).await
    }
}

#[async_trait]
impl<T: Index> Index for &T {
    fn repository_name(&self) -> &RepositoryName {
        (**self).repository_name()
    }

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        (**self).register_workspace(workspace_registration).await
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        (**self).insert_branch(branch).await
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        (**self).update_branch(branch).await
    }

    async fn get_branch(&self, branch_name: &str) -> Result<Branch> {
        (**self).get_branch(branch_name).await
    }

    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>> {
        (**self).list_branches(query).await
    }

    async fn get_commit(&self, commit_id: CommitId) -> Result<Commit> {
        (**self).get_commit(commit_id).await
    }

    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        (**self).list_commits(query).await
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<CommitId> {
        (**self).commit_to_branch(commit, branch).await
    }

    async fn get_tree(&self, id: &str) -> Result<Tree> {
        (**self).get_tree(id).await
    }

    async fn save_tree(&self, tree: &Tree) -> Result<String> {
        (**self).save_tree(tree).await
    }

    async fn lock(&self, lock: &Lock) -> Result<()> {
        (**self).lock(lock).await
    }

    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()> {
        (**self).unlock(lock_domain_id, canonical_path).await
    }

    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock> {
        (**self).get_lock(lock_domain_id, canonical_path).await
    }

    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>> {
        (**self).list_locks(query).await
    }

    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32> {
        (**self).count_locks(query).await
    }
}
