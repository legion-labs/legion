use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use lgn_tracing::span_fn;

use crate::{
    assert_not_locked, find_file_hash_at_commit, make_canonical_relative_path, make_file_read_only,
    make_path_absolute, ChangeType, Workspace,
};

#[derive(Debug, Clone)]
pub struct LocalChange {
    pub relative_path: String,
    pub change_type: ChangeType,
}

impl LocalChange {
    pub fn new(canonical_relative_path: &str, change_type: ChangeType) -> Self {
        Self {
            relative_path: canonical_relative_path.to_lowercase(),
            change_type,
        }
    }
}

#[span_fn]
pub async fn find_local_changes_command() -> Result<Vec<LocalChange>> {
    let workspace = Workspace::find_in_current_directory().await?;

    workspace
        .backend
        .get_local_changes()
        .await
        .map_err(Into::into)
}

pub async fn track_new_file(workspace: &Workspace, path_specified: impl AsRef<Path>) -> Result<()> {
    let abs_path = make_path_absolute(path_specified)?;

    if !abs_path.exists() {
        anyhow::bail!("Error: file {} not found", abs_path.display());
    }
    //todo: make sure the file does not exist in the current tree hierarchy

    let relative_path = make_canonical_relative_path(&workspace.root, &abs_path)?;
    if let Some(change) = workspace
        .backend
        .find_local_change(&relative_path)
        .await
        .context("error searching in local changes")?
    {
        anyhow::bail!(
            "{} already tracked for {:?}",
            change.relative_path,
            change.change_type
        );
    }

    let (_branch_name, current_commit) = workspace.backend.get_current_branch().await?;

    if let Some(_hash) =
        find_file_hash_at_commit(workspace, Path::new(&relative_path), &current_commit).await?
    {
        anyhow::bail!("file already exists in tree");
    }

    assert_not_locked(workspace, &abs_path).await?;
    let local_change = LocalChange::new(&relative_path, ChangeType::Add);

    workspace
        .backend
        .save_local_change(&local_change)
        .await
        .map_err(Into::into)
}

#[span_fn]
pub async fn track_new_file_command(path_specified: impl AsRef<Path>) -> Result<()> {
    let workspace = Workspace::find(path_specified.as_ref()).await?;

    track_new_file(&workspace, path_specified).await
}

pub async fn edit_file(workspace: &Workspace, path_specified: impl AsRef<Path>) -> Result<()> {
    let abs_path = make_path_absolute(path_specified)?;
    fs::metadata(&abs_path)
        .context(format!("error reading metadata for {}", abs_path.display()))?;

    //todo: make sure file is tracked by finding it in the current tree hierarchy
    assert_not_locked(workspace, &abs_path).await?;

    let relative_path = make_canonical_relative_path(&workspace.root, &abs_path)?;

    if let Some(change) = workspace
        .backend
        .find_local_change(&relative_path)
        .await
        .context("error searching in local changes")?
    {
        anyhow::bail!(
            "{} already tracked for {:?}",
            change.relative_path,
            change.change_type
        );
    }

    let local_change = LocalChange::new(&relative_path, ChangeType::Edit);
    workspace.backend.save_local_change(&local_change).await?;
    make_file_read_only(&abs_path, false)
}

#[span_fn]
pub async fn edit_file_command(path_specified: impl AsRef<Path>) -> Result<()> {
    let workspace = Workspace::find(path_specified.as_ref()).await?;

    edit_file(&workspace, path_specified).await
}
