use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sqlx::Connection;

pub struct LocalWorkspaceConnection {
    workspace_path: PathBuf,
    sql_connection: sqlx::AnyConnection,
}

impl LocalWorkspaceConnection {
    pub async fn new(workspace_path: &Path) -> Result<Self> {
        let db_path = workspace_path.join(".lsc/workspace.db3");
        let url = format!("sqlite://{}", db_path.display());

        sqlx::AnyConnection::connect(&url)
            .await
            .context(format!("failed to open database at {}", url))
            .map(|conn| Self {
                workspace_path: workspace_path.to_path_buf(),
                sql_connection: conn,
            })
    }

    pub async fn begin(&mut self) -> Result<sqlx::Transaction<'_, sqlx::Any>> {
        self.sql_connection
            .begin()
            .await
            .context("error beginning transaction on workspace")
    }

    pub fn sql(&mut self) -> &mut sqlx::AnyConnection {
        &mut self.sql_connection
    }

    pub fn workspace_path(&self) -> &Path {
        &self.workspace_path
    }
}
