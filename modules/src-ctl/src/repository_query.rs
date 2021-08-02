use crate::{Branch, Commit, Tree, Workspace};
use async_trait::async_trait;

#[async_trait]
pub trait RepositoryQuery {
    async fn insert_workspace(&self, spec: &Workspace) -> Result<(), String>;
    async fn read_branch(&self, name: &str) -> Result<Branch, String>;
    async fn insert_branch(&self, branch: &Branch) -> Result<(), String>;
    async fn update_branch(&self, branch: &Branch) -> Result<(), String>;
    async fn find_branch(&self, name: &str) -> Result<Option<Branch>, String>;
    async fn find_branches_in_lock_domain(
        &self,
        lock_domain_id: &str,
    ) -> Result<Vec<Branch>, String>;
    async fn read_branches(&self) -> Result<Vec<Branch>, String>;
    async fn read_commit(&self, id: &str) -> Result<Commit, String>;
    async fn insert_commit(&self, commit: &Commit) -> Result<(), String>;
    async fn commit_exists(&self, id: &str) -> Result<bool, String>;
    async fn read_tree(&self, hash: &str) -> Result<Tree, String>;
    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<(), String>;
}
