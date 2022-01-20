use std::path::Path;

use anyhow::{Context, Result};

use crate::{
    assert_not_locked, make_canonical_relative_path, make_file_read_only, make_path_absolute,
    ChangeType, LocalChange, Workspace,
};

pub async fn delete_local_file(
    workspace: &Workspace,
    path_specified: impl AsRef<Path>,
) -> Result<()> {
    let abs_path = make_path_absolute(path_specified)?;

    if !abs_path.exists() {
        anyhow::bail!("file not found: {}", abs_path.display());
    }

    assert_not_locked(workspace, &abs_path).await?;

    let relative_path = make_canonical_relative_path(&workspace.root, &abs_path)?;

    if let Some(change) = workspace
        .backend
        .find_local_change(&relative_path)
        .await
        .context("failed to search for local change")?
    {
        anyhow::bail!(
            "{} is already tracked for {:?}",
            change.relative_path,
            change.change_type
        );
    }

    //todo: lock file
    let local_change = LocalChange::new(&relative_path, ChangeType::Delete);

    workspace.backend.save_local_change(&local_change).await?;
    make_file_read_only(&abs_path, false)?;

    tokio::fs::remove_file(&abs_path)
        .await
        .context(format!("failed to delete file: {}", abs_path.display()))
}

pub async fn delete_file_command(path_specified: impl AsRef<Path>) -> Result<()> {
    let abs_path = make_path_absolute(path_specified.as_ref())?;

    if !abs_path.exists() {
        anyhow::bail!("file not found: {}", abs_path.display());
    }

    let workspace = Workspace::find(&abs_path).await?;
    delete_local_file(&workspace, path_specified).await
}
