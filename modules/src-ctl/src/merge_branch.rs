use crate::{sql::*, *};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::path::Path;

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
) -> Result<(), String> {
    let sql_connection = workspace_connection.sql();
    let sql = "CREATE TABLE branch_merges_pending(name VARCHAR(255) NOT NULL PRIMARY KEY, head VARCHAR(255));";
    if let Err(e) = execute_sql(sql_connection, sql).await {
        return Err(format!("Error creating branch_merges_pending table: {}", e));
    }
    Ok(())
}

pub async fn save_pending_branch_merge(
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    merge_spec: &PendingBranchMerge,
) -> Result<(), String> {
    if let Err(e) = sqlx::query("INSERT OR REPLACE into branch_merges_pending VALUES(?,?);")
        .bind(merge_spec.name.clone())
        .bind(merge_spec.head.clone())
        .execute(workspace_transaction)
        .await
    {
        return Err(format!(
            "Error saving pending branch merge {}: {}",
            merge_spec.name, e
        ));
    }
    Ok(())
}

pub async fn read_pending_branch_merges(
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Vec<PendingBranchMerge>, String> {
    let mut res = Vec::new();
    match sqlx::query(
        "SELECT name, head 
             FROM branch_merges_pending;",
    )
    .fetch_all(transaction)
    .await
    {
        Ok(rows) => {
            for row in rows {
                let merge_pending = PendingBranchMerge {
                    name: row.get("name"),
                    head: row.get("head"),
                };
                res.push(merge_pending);
            }
            Ok(res)
        }
        Err(e) => Err(format!("Error fetching merges pending: {}", e)),
    }
}

pub async fn clear_pending_branch_merges(
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<(), String> {
    let sql = "DELETE from branch_merges_pending;";
    if let Err(e) = execute_sql(transaction, sql).await {
        return Err(format!("Error clearing pending branch merges: {}", e));
    }
    Ok(())
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
) -> Result<String, String> {
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
        if let Err(e) = repo_connection
            .blob_storage()
            .await?
            .download_blob(&local_path, hash_to_sync)
            .await
        {
            return Err(format!(
                "Error downloading {} {}: {}",
                local_path.display(),
                &hash_to_sync,
                e
            ));
        }
        if let Err(e) = make_file_read_only(&local_path, true) {
            return Err(e);
        }
        return Ok(format!("Updated {}", local_path.display()));
    } else {
        //no local file
        if hash_to_sync.is_empty() {
            return Ok(format!("Verified {}", local_path.display()));
        }
        if let Err(e) = repo_connection
            .blob_storage()
            .await?
            .download_blob(&local_path, hash_to_sync)
            .await
        {
            return Err(format!(
                "Error downloading {} {}: {}",
                local_path.display(),
                &hash_to_sync,
                e
            ));
        }
        if let Err(e) = make_file_read_only(&local_path, true) {
            return Err(e);
        }
        track_new_file(
            workspace_root,
            workspace_transaction,
            repo_connection,
            &local_path,
        )
        .await?;
        return Ok(format!("Added {}", local_path.display()));
    }
}

async fn find_commit_ancestors(
    connection: &RepositoryConnection,
    id: &str,
) -> Result<BTreeSet<String>, String> {
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

pub fn merge_branch_command(runtime: &tokio::runtime::Runtime, name: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    let mut workspace_transaction = runtime.block_on(workspace_connection.begin())?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = runtime.block_on(connect_to_server(&workspace_spec))?;
    let query = connection.query();
    let src_branch = runtime.block_on(query.read_branch(name))?;
    let (current_branch_name, current_commit) =
        runtime.block_on(read_current_branch(&mut workspace_transaction))?;
    let old_commit = runtime.block_on(query.read_commit(&current_commit))?;
    let mut destination_branch = runtime.block_on(query.read_branch(&current_branch_name))?;

    let merge_source_ancestors =
        runtime.block_on(find_commit_ancestors(&connection, &src_branch.head))?;
    let src_commit_history = runtime.block_on(find_branch_commits(&connection, &src_branch))?;
    if merge_source_ancestors.contains(&destination_branch.head) {
        //fast forward case
        destination_branch.head = src_branch.head;
        runtime.block_on(update_current_branch(
            &mut workspace_transaction,
            &destination_branch.name,
            &destination_branch.head,
        ))?;
        runtime.block_on(query.update_branch(&destination_branch))?;
        println!("Fast-forward merge: branch updated, synching");
        println!("current commit: {}", current_commit);
        let new_commit = runtime.block_on(query.read_commit(&destination_branch.head))?;

        if let Err(e) = runtime.block_on(workspace_transaction.commit()) {
            return Err(format!(
                "Error in transaction commit for merge_branch_command: {}",
                e
            ));
        }

        return sync_tree_diff(
            runtime,
            &connection,
            &old_commit.root_hash,
            &new_commit.root_hash,
            Path::new(""),
            &workspace_root,
        );
    }

    if current_commit != destination_branch.head {
        return Err(String::from(
            "Workspace not up to date, sync to latest before merge",
        ));
    }

    let mut errors: Vec<String> = Vec::new();
    let destination_commit_history =
        runtime.block_on(find_branch_commits(&connection, &destination_branch))?;
    if let Some(common_ancestor_id) =
        find_latest_common_ancestor(&destination_commit_history, &merge_source_ancestors)
    {
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

        for (path, hash) in to_update.iter() {
            if modified_in_current.contains_key(path) {
                let resolve_pending = ResolvePending::new(
                    path.clone(),
                    common_ancestor_id.clone(),
                    src_branch.head.clone(),
                );
                errors.push(format!("{} conflicts, please resolve before commit", path));
                let full_path = workspace_root.join(path);
                if let Err(e) = runtime.block_on(edit_file(
                    &workspace_root,
                    &mut workspace_transaction,
                    connection.query(),
                    &full_path,
                )) {
                    errors.push(format!("Error editing {}: {}", full_path.display(), e));
                }
                if let Err(e) = runtime.block_on(save_resolve_pending(
                    &mut workspace_transaction,
                    &resolve_pending,
                )) {
                    errors.push(format!("Error saving pending resolve {}: {}", path, e));
                }
            } else {
                match runtime.block_on(change_file_to(
                    &workspace_root,
                    &mut workspace_transaction,
                    &connection,
                    Path::new(path),
                    hash,
                )) {
                    Ok(message) => {
                        println!("{}", message);
                    }
                    Err(e) => {
                        errors.push(e);
                    }
                }
            }
        }
    } else {
        return Err(String::from(
            "Error finding common ancestor for branch merge",
        ));
    }

    //record pending merge to record all parents in upcoming commit
    let pending = PendingBranchMerge::new(&src_branch);
    if let Err(e) = runtime.block_on(save_pending_branch_merge(
        &mut workspace_transaction,
        &pending,
    )) {
        errors.push(e);
    }

    //commit transaction even when errors occur, pending resolves need to be saved
    if let Err(e) = runtime.block_on(workspace_transaction.commit()) {
        return Err(format!(
            "Error in transaction commit for merge_branch_command: {}",
            e
        ));
    }

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    println!("merge completed, ready to commit");
    Ok(())
}
