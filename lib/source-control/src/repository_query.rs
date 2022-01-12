use async_trait::async_trait;

use crate::{
    blob_storage::BlobStorageUrl, Branch, Commit, Error, Lock, Result, Tree, WorkspaceRegistration,
};

#[async_trait]
pub trait RepositoryQuery: Send + Sync {
    async fn ping(&self) -> Result<()>;
    async fn create_repository(
        &self,
        blob_storage_url: Option<BlobStorageUrl>,
    ) -> Result<BlobStorageUrl>;
    async fn destroy_repository(&self) -> Result<()>;
    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()>;
    async fn read_branch(&self, branch_name: &str) -> Result<Branch> {
        self.find_branch(branch_name)
            .await?
            .ok_or_else(|| Error::BranchNotFound {
                branch_name: branch_name.to_string(),
            })
    }
    async fn insert_branch(&self, branch: &Branch) -> Result<()>;
    async fn update_branch(&self, branch: &Branch) -> Result<()>;
    async fn find_branch(&self, branch_name: &str) -> Result<Option<Branch>>;
    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>>;
    async fn read_branches(&self) -> Result<Vec<Branch>>;
    async fn read_commit(&self, commit_id: &str) -> Result<Commit>;
    async fn insert_commit(&self, commit: &Commit) -> Result<()>;
    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()>;
    async fn commit_exists(&self, commit_id: &str) -> Result<bool>;
    async fn read_tree(&self, tree_hash: &str) -> Result<Tree>;
    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<()>;
    async fn insert_lock(&self, lock: &Lock) -> Result<()>;
    async fn find_lock(&self, lock_domain_id: &str, relative_path: &str) -> Result<Option<Lock>>;
    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>>;
    async fn clear_lock(&self, lock_domain_id: &str, relative_path: &str) -> Result<()>;
    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32>;
    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl>;
}
