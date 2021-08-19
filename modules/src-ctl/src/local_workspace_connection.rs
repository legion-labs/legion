use sqlx::Connection;
use std::path::{Path, PathBuf};

pub struct LocalWorkspaceConnection {
    workspace_path: PathBuf,
    sql_connection: sqlx::AnyConnection,
}

impl LocalWorkspaceConnection {
    pub async fn new(workspace_path: &Path) -> Result<Self, String> {
        let db_path = workspace_path.join(".lsc/workspace.db3");
        let url = format!("sqlite://{}", db_path.display());
        match sqlx::AnyConnection::connect(&url).await {
            Err(e) => Err(format!("Error opening database {}: {}", url, e)),
            Ok(c) => Ok(Self {
                workspace_path: workspace_path.to_path_buf(),
                sql_connection: c,
            }),
        }
    }

    pub async fn begin(&mut self) -> Result<sqlx::Transaction<'_, sqlx::Any>, String> {
        match self.sql_connection.begin().await {
            Ok(t) => Ok(t),
            Err(e) => {
                return Err(format!("Error beginning transaction on workspace: {}", e));
            }
        }
    }

    pub fn sql(&mut self) -> &mut sqlx::AnyConnection {
        &mut self.sql_connection
    }

    pub fn workspace_path(&self) -> &Path {
        &self.workspace_path
    }
}
