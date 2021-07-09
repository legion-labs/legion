use futures::executor::block_on;
use sqlx::Connection;
use std::path::{Path, PathBuf};

pub struct LocalWorkspaceConnection {
    workspace_path: PathBuf,
    sql_connection: sqlx::AnyConnection,
}

impl LocalWorkspaceConnection {
    pub fn new(workspace_path: &Path) -> Result<Self, String> {
        let db_path = workspace_path.join(".lsc/workspace.db3");
        let url = format!("sqlite://{}", db_path.display());
        match block_on(sqlx::AnyConnection::connect(&url)) {
            Err(e) => Err(format!("Error opening database {}: {}", url, e)),
            Ok(c) => Ok(Self {
                workspace_path: workspace_path.to_path_buf(),
                sql_connection: c,
            }),
        }
    }

    pub fn sql(&mut self) -> &mut sqlx::AnyConnection {
        &mut self.sql_connection
    }

    pub fn workspace_path(&self) -> &Path {
        &self.workspace_path
    }
}
