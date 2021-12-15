use anyhow::Context;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;

use crate::{
    compute_file_hash, connect_to_server, delete_local_file, edit_file, find_branch_commits,
    find_workspace_root, make_file_read_only, read_current_branch, read_workspace_spec,
    save_resolve_pending, sql::execute_sql, sync_tree_diff, trace_scope, track_new_file,
    update_current_branch, Branch, Commit, LocalWorkspaceConnection, RepositoryConnection,
    ResolvePending,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct PendingBranchMerge {
    pub name: String,
    pub head: String, //commit id
}

impl PendingBranchMerge {
    pub fn new(branch: &Branch) -> Self {
        Self {
            name: branch.name.clone(),
            head: branch.head.clone(),
        }
    }
}

pub async fn init_branch_merge_pending_database(
    workspace_connection: &mut LocalWorkspaceConnection,
) -> Result<()> {
    let sql_connection = workspace_connection.sql();
    let sql = "CREATE TABLE branch_merges_pending(name VARCHAR(255) NOT NULL PRIMARY KEY, head VARCHAR(255));";

    execute_sql(sql_connection, sql)
        .await
        .context("error creating branch_merges_pending table")
}

pub async fn save_pending_branch_merge(
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    merge_spec: &PendingBranchMerge,
) -> Result<()> {
    sqlx::query("INSERT OR REPLACE into branch_merges_pending VALUES(?,?);")
        .bind(merge_spec.name.clone())
        .bind(merge_spec.head.clone())
        .execute(workspace_transaction)
        .await
        .context(format!(
            "error saving pending branch merge {}",
            merge_spec.name
        ))?;

    Ok(())
}

pub async fn read_pending_branch_merges(
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Vec<PendingBranchMerge>> {
    let rows = sqlx::query(
        "SELECT name, head 
             FROM branch_merges_pending;",
    )
    .fetch_all(transaction)
    .await
    .context("error fetching merges pending")?;

    let mut res = Vec::new();

    for row in rows {
        let merge_pending = PendingBranchMerge {
            name: row.get("name"),
            head: row.get("head"),
        };
        res.push(merge_pending);
    }

    Ok(res)
}

pub async fn clear_pending_branch_merges(
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<()> {
    let sql = "DELETE from branch_merges_pending;";

    execute_sql(transaction, sql)
        .await
        .context("error clearing pending branche merges")
}

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
    workspace_root: &Path,
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    repo_connection: &RepositoryConnection,
    relative_path: &Path,
    hash_to_sync: &str,
) -> Result<String> {
    let local_path = workspace_root.join(relative_path);
    if local_path.exists() {
        let local_hash = compute_file_hash(&local_path)?;
        if local_hash == hash_to_sync {
            return Ok(format!("Verified {}", local_path.display()));
        }
        if hash_to_sync.is_empty() {
            delete_local_file(workspace_root, workspace_transaction, &local_path).await?;
            return Ok(format!("Deleted {}", local_path.display()));
        }
        edit_file(
            workspace_root,
            workspace_transaction,
            repo_connection.query(),
            &local_path,
        )
        .await?;

        repo_connection
            .blob_storage()
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
    repo_connection
        .blob_storage()
        .download_blob(&local_path, hash_to_sync)
        .await
        .context(format!(
            "error downloading {} {}",
            local_path.display(),
            &hash_to_sync,
        ))?;

    make_file_read_only(&local_path, true)?;

    track_new_file(
        workspace_root,
        workspace_transaction,
        repo_connection,
        &local_path,
    )
    .await?;

    Ok(format!("Added {}", local_path.display()))
}

async fn find_commit_ancestors(
    connection: &RepositoryConnection,
    id: &str,
) -> Result<BTreeSet<String>> {
    let mut seeds: VecDeque<String> = VecDeque::new();
    seeds.push_back(String::from(id));
    let mut ancestors = BTreeSet::new();
    while !seeds.is_empty() {
        let seed = seeds.pop_front().unwrap();
        let c = connection.query().read_commit(&seed).await?;
        for parent_id in &c.parents {
            if !ancestors.contains(parent_id) {
                ancestors.insert(parent_id.clone());
                seeds.push_back(parent_id.clone());
            }
        }
    }
    Ok(ancestors)
}

#[allow(clippy::too_many_lines)]
pub async fn merge_branch_command(name: &str) -> Result<()> {
    trace_scope!();
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let src_branch = query.read_branch(name).await?;
    let (current_branch_name, current_commit) =
        read_current_branch(&mut workspace_transaction).await?;
    let old_commit = query.read_commit(&current_commit).await?;
    let mut destination_branch = query.read_branch(&current_branch_name).await?;

    let merge_source_ancestors = find_commit_ancestors(&connection, &src_branch.head).await?;
    let src_commit_history = find_branch_commits(&connection, &src_branch).await?;
    if merge_source_ancestors.contains(&destination_branch.head) {
        //fast forward case
        destination_branch.head = src_branch.head;
        update_current_branch(
            &mut workspace_transaction,
            &destination_branch.name,
            &destination_branch.head,
        )
        .await?;
        query.update_branch(&destination_branch).await?;
        println!("Fast-forward merge: branch updated, synching");
        println!("current commit: {}", current_commit);

        let new_commit = query.read_commit(&destination_branch.head).await?;

        workspace_transaction
            .commit()
            .await
            .context("error in transaction commit for merge_branch_command")?;

        return sync_tree_diff(
            Arc::clone(&connection),
            &old_commit.root_hash,
            &new_commit.root_hash,
            Path::new(""),
            &workspace_root,
        )
        .await;
    }

    if current_commit != destination_branch.head {
        anyhow::bail!("workspace not up to date, sync to latest before merge",);
    }

    let destination_commit_history = find_branch_commits(&connection, &destination_branch).await?;

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
                .entry(change.relative_path.clone())
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
                .entry(change.relative_path.clone())
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

            let full_path = workspace_root.join(path);

            if let Err(e) = edit_file(
                &workspace_root,
                &mut workspace_transaction,
                connection.query(),
                &full_path,
            )
            .await
            {
                error_messages.push(format!("Error editing {}: {}", full_path.display(), e));
            }

            if let Err(e) = save_resolve_pending(&mut workspace_transaction, &resolve_pending).await
            {
                error_messages.push(format!("Error saving pending resolve {}: {}", path, e));
            }
        } else {
            match change_file_to(
                &workspace_root,
                &mut workspace_transaction,
                &connection,
                Path::new(path),
                hash,
            )
            .await
            {
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

    if let Err(e) = save_pending_branch_merge(&mut workspace_transaction, &pending).await {
        error_messages.push(e.to_string());
    }

    //commit transaction even when errors occur, pending resolves need to be saved
    workspace_transaction
        .commit()
        .await
        .context("error in transaction commit for merge_branch_command")?;

    if !error_messages.is_empty() {
        anyhow::bail!("the merge was not complete:\n{}", error_messages.join("\n"));
    }

    println!("merge completed, ready to commit");

    Ok(())
}
