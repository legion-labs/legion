use std::path::Path;

use anyhow::{Context, Result};
use lgn_tracing::span_fn;

use crate::{
    fetch_tree_subdir, make_canonical_relative_path, make_file_read_only, make_path_absolute,
    ChangeType, Workspace,
};

#[span_fn]
pub async fn revert_glob_command(pattern: &str) -> Result<()> {
    let mut nb_errors = 0;

    let matcher = glob::Pattern::new(pattern).context("error parsing glob pattern")?;
    let workspace = Workspace::find_in_current_directory().await?;

    for change in workspace.backend.get_local_changes().await? {
        if matcher.matches(&change.relative_path) {
            println!("reverting {}", change.relative_path);
            let local_file_path = workspace.root.join(change.relative_path);
            if let Err(e) = revert_file(&workspace, &local_file_path).await {
                println!("{}", e);
                nb_errors += 1;
            }
        }
    }

    if nb_errors == 0 {
        Ok(())
    } else {
        anyhow::bail!("{} errors", nb_errors)
    }
}

pub async fn revert_file(workspace: &Workspace, path: impl AsRef<Path>) -> Result<()> {
    let abs_path = make_path_absolute(path)?;
    let relative_path = make_canonical_relative_path(&workspace.root, &abs_path)?;
    let local_change = workspace
        .backend
        .find_local_change(&relative_path)
        .await
        .context("error searching in local changes")?
        .ok_or_else(|| anyhow::anyhow!("{} not found in local changes", relative_path))?;

    let parent_dir = Path::new(&relative_path)
        .parent()
        .ok_or(anyhow::anyhow!("no parent to path provided"))?;

    let (_branch_name, current_commit) = workspace.backend.get_current_branch().await?;
    let current_commit = workspace.index_backend.read_commit(&current_commit).await?;
    let root_tree = workspace
        .index_backend
        .read_tree(&current_commit.root_hash)
        .await?;
    let dir_tree = fetch_tree_subdir(workspace, &root_tree, parent_dir).await?;

    if local_change.change_type != ChangeType::Add {
        let file_node = dir_tree
            .find_file_node(
                abs_path
                    .file_name()
                    .expect("no file name in path specified")
                    .to_str()
                    .expect("invalid file name"),
            )
            .ok_or(anyhow::anyhow!("file not found in tree"))?;

        workspace
            .blob_storage
            .download_blob(&abs_path, &file_node.hash)
            .await?;
        make_file_read_only(&abs_path, true)?;
    }
    workspace
        .backend
        .clear_local_changes(&[local_change])
        .await?;

    match workspace
        .backend
        .find_resolve_pending(&relative_path)
        .await
        .context(format!(
            "error searching in resolve pending for {}",
            relative_path
        ))? {
        Some(resolve_pending) => workspace
            .backend
            .clear_resolve_pending(&resolve_pending)
            .await
            .map_err(Into::into),
        None => Ok(()),
    }
}

#[span_fn]
pub async fn revert_file_command(path: impl AsRef<Path>) -> Result<()> {
    let abs_path = make_path_absolute(path.as_ref())?;
    let workspace = Workspace::find(&abs_path).await?;

    revert_file(&workspace, path).await
}
