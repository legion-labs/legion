use std::path::Path;

use anyhow::{Context, Result};
use chrono::prelude::*;
use lgn_tracing::span_fn;
use sha2::{Digest, Sha256};

use crate::{
    assert_not_locked, clear_local_changes, clear_pending_branch_merges, connect_to_server,
    find_workspace_root, make_file_read_only, read_bin_file, read_current_branch,
    read_local_changes, read_pending_branch_merges, read_workspace_spec, update_current_branch,
    update_tree_from_changes, Branch, ChangeType, Commit, HashedChange, LocalChange,
    LocalWorkspaceConnection, RepositoryConnection,
};

async fn upload_localy_edited_blobs(
    workspace_root: &Path,
    repo_connection: &RepositoryConnection,
    local_changes: &[LocalChange],
) -> Result<Vec<HashedChange>> {
    let mut res = Vec::<HashedChange>::new();
    for local_change in local_changes {
        if local_change.change_type == ChangeType::Delete {
            res.push(HashedChange {
                relative_path: local_change.relative_path.clone(),
                hash: String::from(""),
                change_type: local_change.change_type.clone(),
            });
        } else {
            let local_path = workspace_root.join(&local_change.relative_path);
            let local_file_contents = read_bin_file(&local_path)?;
            let hash = format!("{:X}", Sha256::digest(&local_file_contents));
            repo_connection
                .blob_storage()
                .write_blob(&hash, &local_file_contents)
                .await?;
            res.push(HashedChange {
                relative_path: local_change.relative_path.clone(),
                hash: hash.clone(),
                change_type: local_change.change_type.clone(),
            });
        }
    }
    Ok(res)
}

#[span_fn]
fn make_local_files_read_only(workspace_root: &Path, changes: &[HashedChange]) -> Result<()> {
    for change in changes {
        if change.change_type != ChangeType::Delete {
            let full_path = workspace_root.join(&change.relative_path);
            make_file_read_only(&full_path, true)?;
        }
    }
    Ok(())
}

pub async fn commit_local_changes(
    workspace_root: &Path,
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    commit_id: &str,
    message: &str,
) -> Result<()> {
    let workspace_spec = read_workspace_spec(workspace_root)?;
    let (current_branch_name, current_workspace_commit) =
        read_current_branch(workspace_transaction).await?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.index_backend();
    let mut repo_branch = query.read_branch(&current_branch_name).await?;

    if repo_branch.head != current_workspace_commit {
        // Check early to save work, but the real transaction lock will happen later.
        // Don't want to lock too early because a slow client would block everyone.
        anyhow::bail!("workspace is not up to date, aborting commit");
    }

    let local_changes = read_local_changes(workspace_transaction).await?;
    for change in &local_changes {
        let abs_path = workspace_root.join(&change.relative_path);
        assert_not_locked(query, workspace_root, workspace_transaction, &abs_path).await?;
    }
    let hashed_changes =
        upload_localy_edited_blobs(workspace_root, &connection, &local_changes).await?;

    let base_commit = query.read_commit(&current_workspace_commit).await?;

    let new_root_hash = update_tree_from_changes(
        &query.read_tree(&base_commit.root_hash).await?,
        &hashed_changes,
        &connection,
    )
    .await?;

    let mut parent_commits = Vec::from([base_commit.id]);
    for pending_branch_merge in read_pending_branch_merges(workspace_transaction).await? {
        parent_commits.push(pending_branch_merge.head.clone());
    }
    let timestamp = Utc::now();

    let commit = Commit::new(
        String::from(commit_id),
        whoami::username(),
        String::from(message),
        hashed_changes,
        new_root_hash,
        parent_commits,
        timestamp,
    );
    query.commit_to_branch(&commit, &repo_branch).await?;
    repo_branch.head = commit.id.clone();
    update_current_branch(workspace_transaction, &current_branch_name, &commit.id).await?;
    if let Err(e) = make_local_files_read_only(workspace_root, &commit.changes) {
        println!("Error making local files read only: {}", e);
    }
    clear_local_changes(workspace_transaction, &local_changes).await;
    if let Err(e) = clear_pending_branch_merges(workspace_transaction).await {
        println!("{}", e);
    }
    Ok(())
}

pub async fn commit_command(message: &str) -> Result<()> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    let id = uuid::Uuid::new_v4().to_string();

    commit_local_changes(&workspace_root, &mut workspace_transaction, &id, message).await?;

    workspace_transaction
        .commit()
        .await
        .context("error in transaction commit for commit_command")
}

pub async fn find_branch_commits(
    connection: &RepositoryConnection,
    branch: &Branch,
) -> Result<Vec<Commit>> {
    let mut commits = Vec::new();
    let query = connection.index_backend();
    let mut c = query.read_commit(&branch.head).await?;
    commits.push(c.clone());
    while !c.parents.is_empty() {
        let id = &c.parents[0]; //first parent is assumed to be branch trunk
        c = query.read_commit(id).await?;
        commits.push(c.clone());
    }
    Ok(commits)
}
