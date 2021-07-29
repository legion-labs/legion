use crate::{Branch, Commit, Workspace};
use async_trait::async_trait;

#[async_trait]
pub trait RepositoryQuery {
    async fn insert_workspace(&self, spec: &Workspace) -> Result<(), String>;
    async fn read_branch(&self, name: &str) -> Result<Branch, String>;
    async fn read_commit(&self, id: &str) -> Result<Commit, String>;
}
