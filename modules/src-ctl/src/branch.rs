use crate::{sql::execute_sql, *};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

    pub fn from_json(contents: &str) -> Result<Self, String> {
        let parsed: serde_json::Result<Self> = serde_json::from_str(contents);
        match parsed {
            Ok(branch) => Ok(branch),
            Err(e) => Err(format!("Error parsing branch spec {}", e)),
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Error formatting branch {:?}: {}", self.name, e)),
        }
    }
}

pub async fn create_branches_table(sql_connection: &mut sqlx::AnyConnection) -> Result<(), String> {
    let sql = "CREATE TABLE branches(name VARCHAR(255), head VARCHAR(255), parent VARCHAR(255), lock_domain_id VARCHAR(64));
         CREATE UNIQUE INDEX branch_name on branches(name);
        ";
    if let Err(e) = execute_sql(sql_connection, sql).await {
        return Err(format!("Error creating branch table and index: {}", e));
    }
    Ok(())
}

pub async fn create_workspace_branch_table(
    sql_connection: &mut sqlx::AnyConnection,
) -> Result<(), String> {
    let sql = "CREATE TABLE current_branch(name VARCHAR(255), commit_id VARCHAR(255));
        ";
    if let Err(e) = execute_sql(sql_connection, sql).await {
        return Err(format!("Error creating current_branch table: {}", e));
    }
    Ok(())
}

pub async fn insert_current_branch(
    connection: &mut sqlx::AnyConnection,
    branch_name: &str,
    commit_id: &str,
) -> Result<(), String> {
    if let Err(e) = sqlx::query("INSERT INTO current_branch VALUES(?, ?);")
        .bind(branch_name)
        .bind(commit_id)
        .execute(connection)
        .await
    {
        Err(format!("Error inserting into current_branch: {}", e))
    } else {
        Ok(())
    }
}

pub async fn update_current_branch(
    connection: &mut sqlx::AnyConnection,
    branch_name: &str,
    commit_id: &str,
) -> Result<(), String> {
    if let Err(e) = sqlx::query("UPDATE current_branch SET name=?, commit_id=?;")
        .bind(branch_name)
        .bind(commit_id)
        .execute(connection)
        .await
    {
        Err(format!("Error updating current_branch: {}", e))
    } else {
        Ok(())
    }
}

pub async fn read_current_branch(
    connection: &mut sqlx::AnyConnection,
) -> Result<(String, String), String> {
    match sqlx::query(
        "SELECT name, commit_id 
             FROM current_branch;",
    )
    .fetch_one(connection)
    .await
    {
        Ok(row) => {
            let name = row.get("name");
            let commit_id = row.get("commit_id");
            Ok((name, commit_id))
        }
        Err(e) => Err(format!("Error fetching current_branch: {}", e)),
    }
}

pub async fn create_branch_command(name: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let (old_branch_name, old_workspace_commit) =
        read_current_branch(workspace_connection.sql()).await?;
    let old_repo_branch = query.read_branch(&old_branch_name).await?;
    let new_branch = Branch::new(
        String::from(name),
        old_workspace_commit.clone(),
        old_branch_name,
        old_repo_branch.lock_domain_id,
    );
    query.insert_branch(&new_branch).await?;
    update_current_branch(
        workspace_connection.sql(),
        &new_branch.name,
        &new_branch.head,
    )
    .await
}

pub async fn list_branches_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    for branch in query.read_branches().await? {
        println!(
            "{} head:{} parent:{}",
            branch.name, branch.head, branch.parent
        );
    }
    Ok(())
}
