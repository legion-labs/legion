use crate::{sql::execute_sql, *};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Branch {
    pub name: String,
    pub head: String, //commit id
    pub parent: String,
    pub lock_domain_id: String,
}

impl Branch {
    pub fn new(name: String, head: String, parent: String, lock_domain_id: String) -> Self {
        Self {
            name,
            head,
            parent,
            lock_domain_id,
        }
    }
}

pub fn init_branch_database(sql_connection: &mut sqlx::AnyConnection) -> Result<(), String> {
    let sql = "CREATE TABLE branches(name VARCHAR(255), head VARCHAR(255), parent VARCHAR(255), lock_domain_id VARCHAR(64));
         CREATE UNIQUE INDEX branch_name on branches(name);
        ";
    if let Err(e) = execute_sql(sql_connection, sql) {
        return Err(format!("Error creating branch table and index: {}", e));
    }
    Ok(())
}

fn write_branch_spec(file_path: &Path, branch: &Branch) -> Result<(), String> {
    match serde_json::to_string(branch) {
        Ok(json) => write_file(file_path, json.as_bytes()),
        Err(e) => Err(format!("Error formatting branch {:?}: {}", branch, e)),
    }
}

pub fn save_new_branch_to_repo(
    connection: &mut RepositoryConnection,
    branch: &Branch,
) -> Result<(), String> {
    let mut sql_connection = connection.sql();
    if let Err(e) = block_on(
        sqlx::query("INSERT INTO branches VALUES(?, ?, ?, ?);")
            .bind(branch.name.clone())
            .bind(branch.head.clone())
            .bind(branch.parent.clone())
            .bind(branch.lock_domain_id.clone())
            .execute(&mut sql_connection),
    ) {
        return Err(format!("Error inserting into branches: {}", e));
    }
    Ok(())
}

pub fn save_branch_to_repo(
    connection: &mut RepositoryConnection,
    branch: &Branch,
) -> Result<(), String> {
    let mut sql_connection = connection.sql();
    if let Err(e) = block_on(
        sqlx::query(
            "UPDATE branches SET head=?, parent=?, lock_domain_id=?
             WHERE name=?;",
        )
        .bind(branch.head.clone())
        .bind(branch.parent.clone())
        .bind(branch.lock_domain_id.clone())
        .bind(branch.name.clone())
        .execute(&mut sql_connection),
    ) {
        return Err(format!("Error updating branch {}: {}", branch.name, e));
    }
    Ok(())
}
pub fn save_current_branch(workspace_root: &Path, branch: &Branch) -> Result<(), String> {
    let file_path = workspace_root.join(".lsc/branch.json");
    write_branch_spec(&file_path, branch)
}

pub fn read_current_branch(workspace_root: &Path) -> Result<Branch, String> {
    let file_path = workspace_root.join(".lsc/branch.json");
    read_branch(&file_path)
}

pub fn find_branch(
    connection: &mut RepositoryConnection,
    name: &str,
) -> Result<Option<Branch>, String> {
    let mut sql_connection = connection.sql();
    match block_on(
        sqlx::query(
            "SELECT head, parent, lock_domain_id 
             FROM branches
             WHERE name = ?;",
        )
        .bind(name)
        .fetch_optional(&mut sql_connection),
    ) {
        Ok(None) => Ok(None),
        Ok(Some(row)) => {
            let branch = Branch::new(
                String::from(name),
                row.get("head"),
                row.get("parent"),
                row.get("lock_domain_id"),
            );
            Ok(Some(branch))
        }
        Err(e) => Err(format!("Error fetching branch {}: {}", name, e)),
    }
}

pub fn read_branch(branch_file_path: &Path) -> Result<Branch, String> {
    let parsed: serde_json::Result<Branch> =
        serde_json::from_str(&read_text_file(branch_file_path)?);
    match parsed {
        Ok(branch) => Ok(branch),
        Err(e) => Err(format!(
            "Error reading branch spec {}: {}",
            branch_file_path.display(),
            e
        )),
    }
}

pub fn create_branch_command(name: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let mut connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    let old_branch = read_current_branch(&workspace_root)?;
    let new_branch = Branch::new(
        String::from(name),
        old_branch.head.clone(),
        old_branch.name,
        old_branch.lock_domain_id,
    );
    save_new_branch_to_repo(&mut connection, &new_branch)?;
    save_current_branch(&workspace_root, &new_branch)
}

pub fn read_branches(connection: &mut RepositoryConnection) -> Result<Vec<Branch>, String> {
    let mut sql_connection = connection.sql();
    let mut res = Vec::new();
    match block_on(
        sqlx::query(
            "SELECT name, head, parent, lock_domain_id 
             FROM branches;",
        )
        .fetch_all(&mut sql_connection),
    ) {
        Ok(rows) => {
            for r in rows {
                let branch = Branch::new(
                    r.get("name"),
                    r.get("head"),
                    r.get("parent"),
                    r.get("lock_domain_id"),
                );
                res.push(branch);
            }
            Ok(res)
        }
        Err(e) => Err(format!("Error fetching branches: {}", e)),
    }
}

pub fn list_branches_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let mut connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    for branch in read_branches(&mut connection)? {
        println!(
            "{} head:{} parent:{}",
            branch.name, branch.head, branch.parent
        );
    }
    Ok(())
}
