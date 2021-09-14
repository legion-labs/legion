use crate::{sql::*, *};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lock {
    pub relative_path: String, //needs to have a stable representation across platforms because it seeds the hash
    pub lock_domain_id: String,
    pub workspace_id: String,
    pub branch_name: String,
}

pub async fn init_lock_database(sql_connection: &mut sqlx::AnyConnection) -> Result<(), String> {
    let sql = "CREATE TABLE locks(relative_path VARCHAR(512), lock_domain_id VARCHAR(64), workspace_id VARCHAR(255), branch_name VARCHAR(255));
         CREATE UNIQUE INDEX lock_key on locks(relative_path, lock_domain_id);
        ";
    if let Err(e) = execute_sql(sql_connection, sql).await {
        return Err(format!("Error creating locks table and index: {}", e));
    }
    Ok(())
}

pub async fn verify_empty_lock_domain(
    query: &dyn RepositoryQuery,
    lock_domain_id: &str,
) -> Result<(), String> {
    if query.count_locks_in_domain(lock_domain_id).await? > 0 {
        Err(format!("lock domain not empty{}", lock_domain_id))
    } else {
        Ok(())
    }
}

pub async fn lock_file_command(path_specified: &Path) -> Result<(), String> {
    trace_scope!();
    let workspace_root = find_workspace_root(path_specified)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let (branch_name, _current_commit) = read_current_branch(workspace_connection.sql()).await?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let repo_branch = query.read_branch(&branch_name).await?;
    let lock = Lock {
        relative_path: make_canonical_relative_path(&workspace_root, path_specified)?,
        lock_domain_id: repo_branch.lock_domain_id.clone(),
        workspace_id: workspace_spec.id,
        branch_name: repo_branch.name,
    };
    query.insert_lock(&lock).await
}

pub async fn unlock_file_command(path_specified: &Path) -> Result<(), String> {
    trace_scope!();
    let workspace_root = find_workspace_root(path_specified)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let (branch_name, _current_commit) = read_current_branch(workspace_connection.sql()).await?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let repo_branch = query.read_branch(&branch_name).await?;
    let relative_path = make_canonical_relative_path(&workspace_root, path_specified)?;
    query
        .clear_lock(&repo_branch.lock_domain_id, &relative_path)
        .await
}

pub async fn list_locks_command() -> Result<(), String> {
    trace_scope!();
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let (branch_name, _current_commit) = read_current_branch(workspace_connection.sql()).await?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let repo_branch = query.read_branch(&branch_name).await?;
    let locks = query
        .find_locks_in_domain(&repo_branch.lock_domain_id)
        .await?;
    if locks.is_empty() {
        println!("no locks found in domain {}", &repo_branch.lock_domain_id);
    }
    for lock in locks {
        println!(
            "{} in branch {} owned by workspace {}",
            &lock.relative_path, &lock.branch_name, &lock.workspace_id
        );
    }
    Ok(())
}

pub async fn assert_not_locked(
    query: &dyn RepositoryQuery,
    workspace_root: &Path,
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    path_specified: &Path,
) -> Result<(), String> {
    let workspace_spec = read_workspace_spec(workspace_root)?;
    let (current_branch_name, _current_commit) = read_current_branch(transaction).await?;
    let repo_branch = query.read_branch(&current_branch_name).await?;
    let relative_path = make_canonical_relative_path(workspace_root, path_specified)?;
    match query
        .find_lock(&repo_branch.lock_domain_id, &relative_path)
        .await
    {
        Ok(Some(lock)) => {
            if lock.branch_name == current_branch_name && lock.workspace_id == workspace_spec.id {
                Ok(()) //locked by this workspace on this branch - all good
            } else {
                Err(format!(
                    "File {} locked in branch {}, owned by workspace {}",
                    lock.relative_path, lock.branch_name, lock.workspace_id
                ))
            }
        }
        Err(e) => Err(format!(
            "Error validating that {} is lock-free: {}",
            path_specified.display(),
            e
        )),
        Ok(None) => Ok(()),
    }
}
