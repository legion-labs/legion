use async_trait::async_trait;
// use crate::Workspace;

#[async_trait]
pub trait RepositoryQuery {
    //async fn insert_workspace(&self, spec: &Workspace) -> Result<(), String>;

    //todo: remove sql()
    fn sql(&mut self) -> &mut sqlx::AnyConnection;
}
