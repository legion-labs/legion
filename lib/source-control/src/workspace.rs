use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::write_file;
use crate::{
    make_path_absolute, read_text_file, sql::execute_sql, RepositoryAddr, RepositoryConnection,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workspace {
    pub id: String, //a file lock will contain the workspace id
    pub repo_addr: RepositoryAddr,
    pub root: String,
    pub owner: String,
}

pub async fn init_workspace_database(sql_connection: &mut sqlx::AnyConnection) -> Result<()> {
    let sql = "CREATE TABLE workspaces(id VARCHAR(255), root VARCHAR(255), owner VARCHAR(255));
               CREATE UNIQUE INDEX workspace_id on workspaces(id);";

    execute_sql(sql_connection, sql)
        .await
        .context("error creating workspace table and index")
}

pub fn find_workspace_root(directory: &Path) -> Result<PathBuf> {
    if let Ok(_meta) = fs::metadata(directory.join(".lsc")) {
        return Ok(make_path_absolute(directory));
    }

    match directory.parent() {
        None => anyhow::bail!("workspace not found"),
        Some(parent) => find_workspace_root(parent),
    }
}

pub fn read_workspace_spec(workspace_root_dir: &Path) -> Result<Workspace> {
    let workspace_json_path = workspace_root_dir.join(".lsc/workspace.json");

    serde_json::from_str(&read_text_file(&workspace_json_path)?)
        .context("error reading workspace spec")
}

pub fn write_workspace_spec(path: &Path, spec: &Workspace) -> Result<()> {
    let data = serde_json::to_string(spec).context("error serializing workspace spec")?;

    write_file(path, data.as_bytes())
}

pub struct TempPath {
    pub path: PathBuf,
}

impl Drop for TempPath {
    fn drop(&mut self) {
        if self.path.exists() {
            if let Err(e) = fs::remove_file(&self.path) {
                println!("Error deleting temp file {}: {}", self.path.display(), e);
            }
        }
    }
}

pub async fn download_temp_file(
    connection: &RepositoryConnection,
    workspace_root: &Path,
    blob_hash: &str,
) -> Result<TempPath> {
    let tmp_dir = workspace_root.join(".lsc/tmp");
    let temp_file_path = tmp_dir.join(blob_hash);

    connection
        .blob_storage()
        .download_blob(&temp_file_path, blob_hash)
        .await?;

    Ok(TempPath {
        path: temp_file_path,
    })
}
