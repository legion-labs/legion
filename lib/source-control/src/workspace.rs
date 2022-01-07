use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{make_path_absolute, read_text_file, RepositoryConnection};
use crate::{write_file, RepositoryUrl, WorkspaceRegistration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workspace {
    pub registration: WorkspaceRegistration,
    pub repository_url: RepositoryUrl,
    pub root: String,
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

pub async fn download_temp_file(
    connection: &RepositoryConnection,
    workspace_root: &Path,
    blob_hash: &str,
) -> Result<tempfile::TempPath> {
    let tmp_dir = workspace_root.join(".lsc/tmp");
    let temp_file_path = tmp_dir.join(blob_hash);

    connection
        .blob_storage()
        .download_blob(&temp_file_path, blob_hash)
        .await?;

    Ok(tempfile::TempPath::from_path(temp_file_path))
}
