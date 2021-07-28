use crate::Workspace;
use async_trait::async_trait;

#[async_trait]
pub trait RepositoryQuery {
    async fn insert_workspace(&self, spec: &Workspace) -> Result<(), String>;
}
