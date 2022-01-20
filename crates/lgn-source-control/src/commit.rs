use std::path::Path;

use anyhow::Result;
use chrono::prelude::*;
use lgn_tracing::span_fn;
use sha2::{Digest, Sha256};

use crate::{
    assert_not_locked, make_file_read_only, read_bin_file, update_tree_from_changes, Branch,
    ChangeType, Commit, HashedChange, LocalChange, Workspace,
};

async fn upload_localy_edited_blobs(
    workspace: &Workspace,
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
            let local_path = workspace.root.join(&local_change.relative_path);
            let local_file_contents = read_bin_file(&local_path)?;
            let hash = format!("{:X}", Sha256::digest(&local_file_contents));
            workspace
                .blob_storage
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
    workspace: &Workspace,
    commit_id: &str,
    message: &str,
) -> Result<()> {
    // TODO: This used to be done in a transaction, and it probably still should
    // be.
    let (current_branch_name, current_workspace_commit) =
        workspace.backend.get_current_branch().await?;

    let mut repo_branch = workspace
        .index_backend
        .read_branch(&current_branch_name)
        .await?;

    if repo_branch.head != current_workspace_commit {
        // Check early to save work, but the real transaction lock will happen later.
        // Don't want to lock too early because a slow client would block everyone.
        anyhow::bail!("workspace is not up to date, aborting commit");
    }

    let local_changes = workspace.backend.get_local_changes().await?;

    for change in &local_changes {
        let abs_path = workspace.root.join(&change.relative_path);
        assert_not_locked(workspace, &abs_path).await?;
    }

    let hashed_changes = upload_localy_edited_blobs(workspace, &local_changes).await?;

    let base_commit = workspace
        .index_backend
        .read_commit(&current_workspace_commit)
        .await?;

    let new_root_hash = update_tree_from_changes(
        &workspace
            .index_backend
            .read_tree(&base_commit.root_hash)
            .await?,
        &hashed_changes,
        workspace,
    )
    .await?;

    let mut parent_commits = Vec::from([base_commit.id]);

    for pending_branch_merge in workspace.backend.read_pending_branch_merges().await? {
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

    workspace
        .index_backend
        .commit_to_branch(&commit, &repo_branch)
        .await?;
    repo_branch.head = commit.id.clone();
    workspace
        .backend
        .set_current_branch(&current_branch_name, &commit.id)
        .await?;

    if let Err(e) = make_local_files_read_only(&workspace.root, &commit.changes) {
        println!("Error making local files read only: {}", e);
    }

    workspace
        .backend
        .clear_local_changes(&local_changes)
        .await?;
    workspace
        .backend
        .clear_pending_branch_merges()
        .await
        .map_err(Into::into)
}

pub async fn commit_command(message: &str) -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;
    let id = uuid::Uuid::new_v4().to_string();

    commit_local_changes(&workspace, &id, message).await
}

pub async fn find_branch_commits(workspace: &Workspace, branch: &Branch) -> Result<Vec<Commit>> {
    let mut commits = Vec::new();
    let mut c = workspace.index_backend.read_commit(&branch.head).await?;
    commits.push(c.clone());
    while !c.parents.is_empty() {
        let id = &c.parents[0]; //first parent is assumed to be branch trunk
        c = workspace.index_backend.read_commit(id).await?;
        commits.push(c.clone());
    }
    Ok(commits)
}
