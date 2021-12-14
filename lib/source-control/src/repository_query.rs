use anyhow::Result;
use async_trait::async_trait;

use crate::{BlobStorageSpec, Branch, Commit, Lock, Tree, Workspace};

#[async_trait]
pub trait RepositoryQuery {
    async fn insert_workspace(&self, spec: &Workspace) -> Result<()>;
    async fn read_branch(&self, name: &str) -> Result<Branch>;
    async fn insert_branch(&self, branch: &Branch) -> Result<()>;
    async fn update_branch(&self, branch: &Branch) -> Result<()>;
    async fn find_branch(&self, name: &str) -> Result<Option<Branch>>;
    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>>;
    async fn read_branches(&self) -> Result<Vec<Branch>>;
    async fn read_commit(&self, id: &str) -> Result<Commit>;
    async fn insert_commit(&self, commit: &Commit) -> Result<()>;
    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()>;
    async fn commit_exists(&self, id: &str) -> Result<bool>;
    async fn read_tree(&self, hash: &str) -> Result<Tree>;
    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<()>;
    async fn insert_lock(&self, lock: &Lock) -> Result<()>;
    async fn find_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<Option<Lock>>;
    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>>;
    async fn clear_lock(&self, lock_domain_id: &str, canonical_relative_path: &str) -> Result<()>;
    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32>;
    async fn read_blob_storage_spec(&self) -> Result<BlobStorageSpec>;
}
