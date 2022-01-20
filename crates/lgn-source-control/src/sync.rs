use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use lgn_tracing::span_fn;
use sha2::{Digest, Sha256};

use crate::{
    find_file_hash_in_tree, make_file_read_only, read_bin_file, Commit, ResolvePending, Workspace,
};

async fn find_commit_range(
    workspace: &Workspace,
    branch_name: &str,
    start_commit_id: &str,
    end_commit_id: &str,
) -> Result<Vec<Commit>> {
    let repo_branch = workspace.index_backend.read_branch(branch_name).await?;
    let mut current_commit = workspace
        .index_backend
        .read_commit(&repo_branch.head)
        .await?;
    while current_commit.id != start_commit_id && current_commit.id != end_commit_id {
        if current_commit.parents.is_empty() {
            anyhow::bail!(
                "commits {} and {} not found in branch {}",
                start_commit_id,
                end_commit_id,
                branch_name
            );
        }
        current_commit = workspace
            .index_backend
            .read_commit(&current_commit.parents[0])
            .await?;
    }
    let mut commits = vec![current_commit.clone()];
    if current_commit.id == start_commit_id && current_commit.id == end_commit_id {
        return Ok(commits);
    }
    current_commit = workspace
        .index_backend
        .read_commit(&current_commit.parents[0])
        .await?;
    commits.push(current_commit.clone());
    while current_commit.id != start_commit_id && current_commit.id != end_commit_id {
        if current_commit.parents.is_empty() {
            anyhow::bail!(
                "commit {} or {} not found in branch {}",
                &start_commit_id,
                &end_commit_id,
                branch_name
            );
        }
        current_commit = workspace
            .index_backend
            .read_commit(&current_commit.parents[0])
            .await?;
        commits.push(current_commit.clone());
    }
    Ok(commits)
}

pub fn compute_file_hash(p: &Path) -> Result<String> {
    let contents = read_bin_file(p)?;
    let hash = format!("{:X}", Sha256::digest(&contents));
    Ok(hash)
}

pub async fn sync_file(
    workspace: &Workspace,
    local_path: PathBuf,
    hash_to_sync: &str,
) -> Result<String> {
    let local_hash = if local_path.exists() {
        compute_file_hash(&local_path)?
    } else {
        String::new()
    };
    if local_hash == hash_to_sync {
        return Ok(format!("Verified {}", local_path.display()));
    }
    if let Ok(meta) = fs::metadata(&local_path) {
        let mut permissions = meta.permissions();
        if !permissions.readonly() {
            anyhow::bail!(
                "local file {} is writable: skipping sync for this file",
                local_path.display()
            );
        }

        permissions.set_readonly(false);
        fs::set_permissions(&local_path, permissions)
            .context(format!("failed to set {} read-write", local_path.display()))?;

        if hash_to_sync.is_empty() {
            fs::remove_file(&local_path)
                .context(format!("failed to remove {}", local_path.display()))?;

            return Ok(format!("Deleted {}", local_path.display()));
        }
        workspace
            .blob_storage
            .download_blob(&local_path, hash_to_sync)
            .await
            .context(format!(
                "failed to download {} ({})",
                local_path.display(),
                &hash_to_sync
            ))?;

        make_file_read_only(&local_path, true)?;

        return Ok(format!("Updated {}", local_path.display()));
    }

    //there is no local file, downloading a fresh copy
    let parent_dir = local_path.parent().unwrap();

    if !parent_dir.exists() {
        std::fs::create_dir_all(parent_dir).context(format!(
            "failed to create parent directory: {}",
            parent_dir.display()
        ))?;
    }

    workspace
        .blob_storage
        .download_blob(&local_path, hash_to_sync)
        .await
        .context(format!(
            "failed to download {} ({})",
            local_path.display(),
            hash_to_sync
        ))?;

    make_file_read_only(&local_path, true)?;

    return Ok(format!("Added {}", local_path.display()));
}

pub async fn sync_workspace(
    workspace: &Workspace,
    branch_name: &str,
    current_commit: &str,
    destination_commit: &str,
) -> Result<()> {
    let commits =
        find_commit_range(workspace, branch_name, current_commit, destination_commit).await?;

    let mut to_download: BTreeMap<String, String> = BTreeMap::new();
    let (_last, commits_to_process) = commits.split_last().unwrap();

    if commits[0].id == destination_commit {
        //sync forwards
        for commit in commits_to_process {
            for change in &commit.changes {
                to_download
                    .entry(change.relative_path.clone())
                    .or_insert_with(|| change.hash.clone());
            }
        }
    } else {
        //sync backwards is slower and could be optimized if we had before&after hashes
        // in changes
        let ref_commit = &commits.last().unwrap();
        assert!(ref_commit.id == destination_commit);
        let root_tree = workspace
            .index_backend
            .read_tree(&ref_commit.root_hash)
            .await?;

        let mut to_update: BTreeSet<String> = BTreeSet::new();

        for commit in commits_to_process {
            for change in &commit.changes {
                to_update.insert(change.relative_path.clone());
            }
        }

        for path in to_update {
            to_download.insert(
                path.clone(),
                //todo: find_file_hash_in_tree should flag NotFound as a distinct case and we
                // should fail on error
                match find_file_hash_in_tree(workspace, Path::new(&path), &root_tree).await {
                    Ok(Some(hash)) => hash,
                    Ok(None) => String::new(),
                    Err(e) => {
                        return Err(e);
                    }
                },
            );
        }
    }

    let local_changes_map: HashMap<_, _> = workspace
        .backend
        .get_local_changes()
        .await
        .context("failed to read local changes")?
        .into_iter()
        .map(|change| (change.relative_path.clone(), change))
        .collect();

    let mut errors: Vec<String> = Vec::new();

    for (relative_path, latest_hash) in to_download {
        if let Some(_change) = local_changes_map.get(&relative_path) {
            println!(
                "{} changed locally, recording pending merge and leaving the local file untouched",
                relative_path
            );

            //todo: handle case where merge pending already exists
            //todo: validate how we want to deal with merge pending with syncing backwards
            let merge_pending = ResolvePending::new(
                relative_path.clone(),
                String::from(current_commit),
                String::from(destination_commit),
            );

            if let Err(e) = workspace.backend.save_resolve_pending(&merge_pending).await {
                errors.push(format!(
                    "Error saving pending merge {}: {}",
                    relative_path, e
                ));
            }
        } else {
            //no local change, ok to sync
            let local_path = workspace.root.join(relative_path);

            match sync_file(workspace, local_path, &latest_hash).await {
                Ok(message) => {
                    println!("{}", message);
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("{}", errors.join("\n"));
    }

    Ok(())
}

#[span_fn]
pub async fn sync_to_command(commit_id: &str) -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;
    let (current_branch_name, current_commit) = workspace.backend.get_current_branch().await?;

    let mut errors: Vec<String> = Vec::new();

    if let Err(e) =
        sync_workspace(&workspace, &current_branch_name, &current_commit, commit_id).await
    {
        errors.push(e.to_string());
    }

    if let Err(e) = workspace
        .backend
        .set_current_branch(&current_branch_name, commit_id)
        .await
    {
        errors.push(e.to_string());
    }

    if !errors.is_empty() {
        anyhow::bail!("{}", errors.join("\n"));
    }

    Ok(())
}

#[span_fn]
pub async fn sync_command() -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;
    let (branch_name, _current_commit) = workspace.backend.get_current_branch().await?;
    let repo_branch = workspace.index_backend.read_branch(&branch_name).await?;

    sync_to_command(&repo_branch.head).await
}
