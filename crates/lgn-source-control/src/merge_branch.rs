use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use lgn_tracing::span_fn;
use serde::{Deserialize, Serialize};

use crate::{
    compute_file_hash, delete_local_file, find_branch_commits, make_file_read_only, sync_tree_diff,
    Branch, Commit, ResolvePending, Workspace,
};

fn find_latest_common_ancestor(
    sequence_branch_one: &[Commit],
    set_branch_two: &BTreeSet<String>,
) -> Option<String> {
    // if the times are reliable we can cut short this search
    for c in sequence_branch_one {
        if set_branch_two.contains(&c.id) {
            return Some(c.id.clone());
        }
    }
    None
}

async fn change_file_to(
    workspace: &Workspace,
    relative_path: &Path,
    hash_to_sync: &str,
) -> Result<String> {
    let local_path = workspace.root.join(relative_path);
    if local_path.exists() {
        let local_hash = compute_file_hash(&local_path)?;
        if local_hash == hash_to_sync {
            return Ok(format!("Verified {}", local_path.display()));
        }
        if hash_to_sync.is_empty() {
            delete_local_file(workspace, &local_path).await?;
            return Ok(format!("Deleted {}", local_path.display()));
        }
        workspace.edit_files(&[&local_path]).await?;

        workspace
            .blob_storage
            .download_blob(&local_path, hash_to_sync)
            .await
            .context(format!(
                "error downloading {} {}",
                local_path.display(),
                &hash_to_sync,
            ))?;

        if let Err(e) = make_file_read_only(&local_path, true) {
            return Err(e);
        }
        return Ok(format!("Updated {}", local_path.display()));
    }
    //no local file
    if hash_to_sync.is_empty() {
        return Ok(format!("Verified {}", local_path.display()));
    }
    workspace
        .blob_storage
        .download_blob(&local_path, hash_to_sync)
        .await
        .context(format!(
            "error downloading {} {}",
            local_path.display(),
            &hash_to_sync,
        ))?;

    make_file_read_only(&local_path, true)?;

    workspace.add_files(&[&local_path]).await?;

    Ok(format!("Added {}", local_path.display()))
}

async fn find_commit_ancestors(workspace: &Workspace, id: &str) -> Result<BTreeSet<String>> {
    let mut seeds: VecDeque<String> = VecDeque::new();
    seeds.push_back(String::from(id));
    let mut ancestors = BTreeSet::new();
    while !seeds.is_empty() {
        let seed = seeds.pop_front().unwrap();
        let c = workspace.index_backend.read_commit(&seed).await?;
        for parent_id in &c.parents {
            if !ancestors.contains(parent_id) {
                ancestors.insert(parent_id.clone());
                seeds.push_back(parent_id.clone());
            }
        }
    }
    Ok(ancestors)
}

#[span_fn]
pub async fn merge_branch_command(name: &str) -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;
    let src_branch = workspace.index_backend.read_branch(name).await?;
    let (current_branch_name, current_commit) = workspace.backend.get_current_branch().await?;
    let old_commit = workspace.index_backend.read_commit(&current_commit).await?;
    let mut destination_branch = workspace
        .index_backend
        .read_branch(&current_branch_name)
        .await?;

    let merge_source_ancestors = find_commit_ancestors(&workspace, &src_branch.head).await?;
    let src_commit_history = find_branch_commits(&workspace, &src_branch).await?;
    if merge_source_ancestors.contains(&destination_branch.head) {
        //fast forward case
        destination_branch.head = src_branch.head;
        workspace
            .backend
            .set_current_branch(&destination_branch.name, &destination_branch.head)
            .await?;
        workspace
            .index_backend
            .update_branch(&destination_branch)
            .await?;
        println!("Fast-forward merge: branch updated, synching");
        println!("current commit: {}", current_commit);

        let new_commit = workspace
            .index_backend
            .read_commit(&destination_branch.head)
            .await?;

        return sync_tree_diff(
            &workspace,
            &old_commit.root_tree_id,
            &new_commit.root_tree_id,
            Path::new(""),
        )
        .await;
    }

    if current_commit != destination_branch.head {
        anyhow::bail!("workspace not up to date, sync to latest before merge",);
    }

    let destination_commit_history = find_branch_commits(&workspace, &destination_branch).await?;

    let common_ancestor_id =
        find_latest_common_ancestor(&destination_commit_history, &merge_source_ancestors).ok_or(
            anyhow::format_err!("could not find common ancestor for branch merge"),
        )?;

    let mut modified_in_current: BTreeMap<String, String> = BTreeMap::new();

    for commit in &destination_commit_history {
        if commit.id == common_ancestor_id {
            break;
        }
        for change in &commit.changes {
            modified_in_current
                .entry(change.canonical_path.clone())
                .or_insert_with(|| change.hash.clone());
        }
    }

    let mut to_update: BTreeMap<String, String> = BTreeMap::new();

    for commit in &src_commit_history {
        if commit.id == common_ancestor_id {
            break;
        }
        for change in &commit.changes {
            to_update
                .entry(change.canonical_path.clone())
                .or_insert_with(|| change.hash.clone());
        }
    }

    let mut error_messages: Vec<String> = Vec::new();

    for (path, hash) in &to_update {
        if modified_in_current.contains_key(path) {
            let resolve_pending = ResolvePending::new(
                path.clone(),
                common_ancestor_id.clone(),
                src_branch.head.clone(),
            );

            error_messages.push(format!("{} conflicts, please resolve before commit", path));

            let full_path = workspace.root.join(path);

            if let Err(e) = workspace.edit_files(&[&full_path]).await {
                error_messages.push(format!("Error editing {}: {}", full_path.display(), e));
            }

            if let Err(e) = workspace
                .backend
                .save_resolve_pending(&resolve_pending)
                .await
            {
                error_messages.push(format!("Error saving pending resolve {}: {}", path, e));
            }
        } else {
            match change_file_to(&workspace, Path::new(path), hash).await {
                Ok(message) => {
                    println!("{}", message);
                }
                Err(e) => {
                    error_messages.push(e.to_string());
                }
            }
        }
    }

    //record pending merge to record all parents in upcoming commit
    let pending = PendingBranchMerge::new(&src_branch);

    if let Err(e) = workspace.backend.save_pending_branch_merge(&pending).await {
        error_messages.push(e.to_string());
    }

    if !error_messages.is_empty() {
        anyhow::bail!("the merge was not complete:\n{}", error_messages.join("\n"));
    }

    println!("merge completed, ready to commit");

    Ok(())
}
