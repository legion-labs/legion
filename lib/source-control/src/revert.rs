use anyhow::{Context, Result};
use std::path::Path;

use crate::{
    clear_local_change, clear_resolve_pending, connect_to_server, fetch_tree_subdir,
    find_local_change, find_resolve_pending, find_workspace_root, make_canonical_relative_path,
    make_file_read_only, make_path_absolute, read_current_branch, read_local_changes,
    read_workspace_spec, trace_scope, ChangeType, LocalWorkspaceConnection, RepositoryConnection,
};

pub async fn revert_glob_command(pattern: &str) -> Result<()> {
    trace_scope!();
    let mut nb_errors = 0;

    let matcher = glob::Pattern::new(pattern).context("error parsing glob pattern")?;
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;

    for change in read_local_changes(&mut workspace_transaction).await? {
        if matcher.matches(&change.relative_path) {
            println!("reverting {}", change.relative_path);
            let local_file_path = workspace_root.join(change.relative_path);
            if let Err(e) =
                revert_file(&mut workspace_transaction, &connection, &local_file_path).await
            {
                println!("{}", e);
                nb_errors += 1;
            }
        }
    }

    workspace_transaction
        .commit()
        .await
        .context("error in transaction commit for revert_glob_command")?;

    if nb_errors == 0 {
        Ok(())
    } else {
        anyhow::bail!("{} errors", nb_errors)
    }
}

pub async fn revert_file(
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    repo_connection: &RepositoryConnection,
    path: &Path,
) -> Result<()> {
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let relative_path = make_canonical_relative_path(&workspace_root, &abs_path)?;
    let local_change = find_local_change(workspace_transaction, &relative_path)
        .await
        .context("error searching in local changes")?
        .ok_or_else(|| anyhow::anyhow!("{} not found in local changes", relative_path))?;

    let parent_dir = Path::new(&relative_path)
        .parent()
        .ok_or(anyhow::anyhow!("no parent to path provided"))?;

    let (_branch_name, current_commit) = read_current_branch(workspace_transaction).await?;
    let query = repo_connection.query();
    let current_commit = query.read_commit(&current_commit).await?;
    let root_tree = query.read_tree(&current_commit.root_hash).await?;
    let dir_tree = fetch_tree_subdir(query, &root_tree, parent_dir).await?;

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

        repo_connection
            .blob_storage()
            .await?
            .download_blob(&abs_path, &file_node.hash)
            .await?;
        make_file_read_only(&abs_path, true)?;
    }
    clear_local_change(workspace_transaction, &local_change).await?;
    match find_resolve_pending(workspace_transaction, &relative_path)
        .await
        .context(format!(
            "error searching in resolve pending for {}",
            relative_path
        ))? {
        Some(resolve_pending) => {
            clear_resolve_pending(workspace_transaction, &resolve_pending).await
        }
        None => Ok(()),
    }
}

pub async fn revert_file_command(path: &Path) -> Result<()> {
    trace_scope!();
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let repo_connection = connect_to_server(&workspace_spec).await?;

    revert_file(&mut workspace_transaction, &repo_connection, path).await?;

    workspace_transaction
        .commit()
        .await
        .context("error in transaction commit for revert_file_command")
}
