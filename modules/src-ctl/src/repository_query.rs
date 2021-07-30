use crate::{Branch, Commit, Tree, Workspace};
use async_trait::async_trait;

#[async_trait]
pub trait RepositoryQuery {
    async fn insert_workspace(&self, spec: &Workspace) -> Result<(), String>;
    async fn read_branch(&self, name: &str) -> Result<Branch, String>;
    async fn read_commit(&self, id: &str) -> Result<Commit, String>;
    async fn read_tree(&self, hash: &str) -> Result<Tree, String>;
    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<(), String>;
}
