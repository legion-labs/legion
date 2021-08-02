use crate::{sql::*, *};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lock {
    pub relative_path: String, //needs to have a stable representation across platforms because it seeds the hash
    pub lock_domain_id: String,
    pub workspace_id: String,
    pub branch_name: String,
}

pub fn init_lock_database(sql_connection: &mut sqlx::AnyConnection) -> Result<(), String> {
    let sql = "CREATE TABLE locks(relative_path VARCHAR(512), lock_domain_id VARCHAR(64), workspace_id VARCHAR(255), branch_name VARCHAR(255));
         CREATE UNIQUE INDEX lock_key on locks(relative_path, lock_domain_id);
        ";
    if let Err(e) = execute_sql(sql_connection, sql) {
        return Err(format!("Error creating locks table and index: {}", e));
    }
    Ok(())
}

pub fn clear_lock(
    connection: &RepositoryConnection,
    lock_domain_id: &str,
    canonical_relative_path: &str,
) -> Result<(), String> {
    let mut sql_connection = connection.sql();
    if let Err(e) = block_on(
        sqlx::query("DELETE from locks WHERE relative_path=? AND lock_domain_id=?;")
            .bind(canonical_relative_path)
            .bind(lock_domain_id)
            .execute(&mut sql_connection),
    ) {
        return Err(format!("Error clearing lock: {}", e));
    }
    Ok(())
}

pub fn verify_empty_lock_domain(
    connection: &RepositoryConnection,
    lock_domain_id: &str,
) -> Result<(), String> {
    let mut sql_connection = connection.sql();
    match block_on(
        sqlx::query(
            "SELECT count(*) as count
             FROM locks
             WHERE lock_domain_id = ?;",
        )
        .bind(lock_domain_id)
        .fetch_one(&mut sql_connection),
    ) {
        Err(e) => Err(format!("Error counting locks: {}", e)),
        Ok(row) => {
            let count: i32 = row.get("count");
            if count > 0 {
                Err(format!("lock domain not empty{}", lock_domain_id))
            } else {
                Ok(())
            }
        }
    }
}

pub fn read_locks(
    connection: &RepositoryConnection,
    lock_domain_id: &str,
) -> Result<Vec<Lock>, String> {
    let mut sql_connection = connection.sql();
    match block_on(
        sqlx::query(
            "SELECT relative_path, workspace_id, branch_name
             FROM locks
             WHERE lock_domain_id=?;",
        )
        .bind(lock_domain_id)
        .fetch_all(&mut sql_connection),
    ) {
        Ok(rows) => {
            let mut locks = Vec::new();
            for r in rows {
                locks.push(Lock {
                    relative_path: r.get("relative_path"),
                    lock_domain_id: String::from(lock_domain_id),
                    workspace_id: r.get("workspace_id"),
                    branch_name: r.get("branch_name"),
                });
            }
            Ok(locks)
        }
        Err(e) => Err(format!("Error listing locks: {}", e)),
    }
}

pub async fn lock_file_command(path_specified: &Path) -> Result<(), String> {
    let workspace_root = find_workspace_root(path_specified)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let repo_branch = query.read_branch(&current_branch.name).await?;
    let lock = Lock {
        relative_path: make_canonical_relative_path(&workspace_root, path_specified)?,
        lock_domain_id: repo_branch.lock_domain_id.clone(),
        workspace_id: workspace_spec.id,
        branch_name: repo_branch.name,
    };
    query.insert_lock(&lock).await
}

pub fn unlock_file_command(path_specified: &Path) -> Result<(), String> {
    let workspace_root = find_workspace_root(path_specified)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    let query = connection.query();
    let repo_branch = tokio_runtime.block_on(query.read_branch(&current_branch.name))?;
    let relative_path = make_canonical_relative_path(&workspace_root, path_specified)?;
    clear_lock(&connection, &repo_branch.lock_domain_id, &relative_path)
}

pub fn list_locks_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    let query = connection.query();
    let repo_branch = tokio_runtime.block_on(query.read_branch(&current_branch.name))?;
    let locks = read_locks(&connection, &repo_branch.lock_domain_id)?;
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
    path_specified: &Path,
) -> Result<(), String> {
    let workspace_spec = read_workspace_spec(workspace_root)?;
    let current_branch = read_current_branch(workspace_root)?;
    let repo_branch = query.read_branch(&current_branch.name).await?;
    let relative_path = make_canonical_relative_path(workspace_root, path_specified)?;
    match query
        .find_lock(&repo_branch.lock_domain_id, &relative_path)
        .await
    {
        Ok(Some(lock)) => {
            if lock.branch_name == current_branch.name && lock.workspace_id == workspace_spec.id {
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
