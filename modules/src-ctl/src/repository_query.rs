use crate::{Branch, Workspace};
use async_trait::async_trait;

#[async_trait]
pub trait RepositoryQuery {
    async fn insert_workspace(&self, spec: &Workspace) -> Result<(), String>;
    async fn read_branch(&self, name: &str) -> Result<Branch, String>;
}
