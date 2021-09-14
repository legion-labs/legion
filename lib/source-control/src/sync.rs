use crate::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

async fn find_commit_range(
    connection: &RepositoryConnection,
    branch_name: &str,
    start_commit_id: &str,
    end_commit_id: &str,
) -> Result<Vec<Commit>, String> {
    let query = connection.query();
    let repo_branch = query.read_branch(branch_name).await?;
    let mut current_commit = query.read_commit(&repo_branch.head).await?;
    while current_commit.id != start_commit_id && current_commit.id != end_commit_id {
        if current_commit.parents.is_empty() {
            return Err(format!(
                "commits {} and {} not found in branch {}",
                start_commit_id, end_commit_id, branch_name
            ));
        }
        current_commit = query.read_commit(&current_commit.parents[0]).await?;
    }
    let mut commits = vec![current_commit.clone()];
    if current_commit.id == start_commit_id && current_commit.id == end_commit_id {
        return Ok(commits);
    }
    current_commit = query.read_commit(&current_commit.parents[0]).await?;
    commits.push(current_commit.clone());
    while current_commit.id != start_commit_id && current_commit.id != end_commit_id {
        if current_commit.parents.is_empty() {
            return Err(format!(
                "commit {} or {} not found in branch {}",
                &start_commit_id, &end_commit_id, branch_name
            ));
        }
        current_commit = query.read_commit(&current_commit.parents[0]).await?;
        commits.push(current_commit.clone());
    }
    Ok(commits)
}

pub fn compute_file_hash(p: &Path) -> Result<String, String> {
    let contents = read_bin_file(p)?;
    let hash = format!("{:X}", Sha256::digest(&contents));
    Ok(hash)
}

pub async fn sync_file(
    connection: &RepositoryConnection,
    local_path: &Path,
    hash_to_sync: &str,
) -> Result<String, String> {
    let local_hash = if local_path.exists() {
        compute_file_hash(local_path)?
    } else {
        String::new()
    };
    if local_hash == hash_to_sync {
        return Ok(format!("Verified {}", local_path.display()));
    }
    match fs::metadata(&local_path) {
        Ok(meta) => {
            let mut permissions = meta.permissions();
            if !permissions.readonly() {
                return Err(format!(
                    "Error: local file {} is writable. Skipping sync for this file.",
                    local_path.display()
                ));
            }

            permissions.set_readonly(false);
            if let Err(e) = fs::set_permissions(&local_path, permissions) {
                return Err(format!(
                    "Error making file {} writable: {}",
                    local_path.display(),
                    e
                ));
            }

            if hash_to_sync.is_empty() {
                if let Err(e) = fs::remove_file(&local_path) {
                    return Err(format!("Error deleting {}: {}", local_path.display(), e));
                } else {
                    return Ok(format!("Deleted {}", local_path.display()));
                }
            }
            if let Err(e) = connection
                .blob_storage()
                .await?
                .download_blob(local_path, hash_to_sync)
                .await
            {
                return Err(format!(
                    "Error downloading {} {}: {}",
                    local_path.display(),
                    &hash_to_sync,
                    e
                ));
            }
            if let Err(e) = make_file_read_only(local_path, true) {
                return Err(e);
            }
            return Ok(format!("Updated {}", local_path.display()));
        }
        Err(_) => {
            //there is no local file, downloading a fresh copy
            let parent_dir = local_path.parent().unwrap();
            if !parent_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(parent_dir) {
                    return Err(format!(
                        "Error creating directory path {}: {}",
                        parent_dir.display(),
                        e
                    ));
                }
            }
            if let Err(e) = connection
                .blob_storage()
                .await?
                .download_blob(local_path, hash_to_sync)
                .await
            {
                return Err(format!(
                    "Error downloading {} {}: {}",
                    local_path.display(),
                    &hash_to_sync,
                    e
                ));
            }
            if let Err(e) = make_file_read_only(local_path, true) {
                return Err(e);
            }
            return Ok(format!("Added {}", local_path.display()));
        }
    }
}

pub async fn sync_workspace(
    workspace_root: &Path,
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    connection: &RepositoryConnection,
    branch_name: &str,
    current_commit: &str,
    destination_commit: &str,
) -> Result<(), String> {
    let commits =
        find_commit_range(connection, branch_name, current_commit, destination_commit).await?;
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
        //sync backwards is slower and could be optimized if we had before&after hashes in changes
        let ref_commit = &commits.last().unwrap();
        assert!(ref_commit.id == destination_commit);
        let root_tree = connection.query().read_tree(&ref_commit.root_hash).await?;

        let mut to_update: BTreeSet<String> = BTreeSet::new();
        for commit in commits_to_process {
            for change in &commit.changes {
                to_update.insert(change.relative_path.clone());
            }
        }
        for path in to_update {
            to_download.insert(
                path.clone(),
                //todo: find_file_hash_in_tree should flag NotFound as a distinct case and we should fail on error
                match find_file_hash_in_tree(connection, Path::new(&path), &root_tree).await {
                    Ok(Some(hash)) => hash,
                    Ok(None) => String::new(),
                    Err(e) => {
                        return Err(e);
                    }
                },
            );
        }
    }

    let mut local_changes_map = HashMap::new();
    match read_local_changes(workspace_transaction).await {
        Ok(changes_vec) => {
            for change in changes_vec {
                local_changes_map.insert(change.relative_path.clone(), change.clone());
            }
        }
        Err(e) => {
            return Err(format!("Error reading local changes: {}", e));
        }
    }

    let mut errors: Vec<String> = Vec::new();
    for (relative_path, latest_hash) in to_download {
        match local_changes_map.get(&relative_path) {
            Some(_change) => {
                println!("{} changed locally, recording pending merge and leaving the local file untouched", relative_path);
                //todo: handle case where merge pending already exists
                //todo: validate how we want to deal with merge pending with syncing backwards
                let merge_pending = ResolvePending::new(
                    relative_path.clone(),
                    String::from(current_commit),
                    String::from(destination_commit),
                );
                if let Err(e) = save_resolve_pending(workspace_transaction, &merge_pending).await {
                    errors.push(format!(
                        "Error saving pending merge {}: {}",
                        relative_path, e
                    ));
                }
            }
            None => {
                //no local change, ok to sync
                let local_path = workspace_root.join(relative_path);
                match sync_file(connection, &local_path, &latest_hash).await {
                    Ok(message) => {
                        println!("{}", message);
                    }
                    Err(e) => {
                        errors.push(e);
                    }
                }
            }
        }
    }
    if !errors.is_empty() {
        let message = errors.join("\n");
        return Err(message);
    }
    Ok(())
}

pub async fn sync_to_command(commit_id: &str) -> Result<(), String> {
    trace_scope!();
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let (current_branch_name, current_commit) =
        read_current_branch(&mut workspace_transaction).await?;

    let mut errors: Vec<String> = Vec::new();
    if let Err(e) = sync_workspace(
        &workspace_root,
        &mut workspace_transaction,
        &connection,
        &current_branch_name,
        &current_commit,
        commit_id,
    )
    .await
    {
        errors.push(e);
    }

    if let Err(e) =
        update_current_branch(&mut workspace_transaction, &current_branch_name, commit_id).await
    {
        errors.push(e);
    }

    if let Err(e) = workspace_transaction.commit().await {
        errors.push(format!(
            "Error in transaction commit for sync_to_command: {}",
            e
        ));
    }

    if !errors.is_empty() {
        let message = errors.join("\n");
        return Err(message);
    }
    Ok(())
}

pub async fn sync_command() -> Result<(), String> {
    trace_scope!();
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let (branch_name, _current_commit) = read_current_branch(workspace_connection.sql()).await?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let repo_branch = query.read_branch(&branch_name).await?;
    sync_to_command(&repo_branch.head).await
}
