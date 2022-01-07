use anyhow::{Context, Result};
use lgn_tracing::span_fn;
use sqlx::Row;

use crate::{
    connect_to_server, find_workspace_root, read_workspace_spec, sql::execute_sql,
    LocalWorkspaceConnection,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    pub name: String,
    pub head: String, //commit id
    pub parent: String,
    pub lock_domain_id: String,
}

impl From<Branch> for lgn_source_control_proto::Branch {
    fn from(branch: Branch) -> Self {
        Self {
            name: branch.name,
            head: branch.head,
            parent: branch.parent,
            lock_domain_id: branch.lock_domain_id,
        }
    }
}

impl From<lgn_source_control_proto::Branch> for Branch {
    fn from(branch: lgn_source_control_proto::Branch) -> Self {
        Self {
            name: branch.name,
            head: branch.head,
            parent: branch.parent,
            lock_domain_id: branch.lock_domain_id,
        }
    }
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

pub async fn create_branches_table(sql_connection: &mut sqlx::AnyConnection) -> Result<()> {
    let sql = "CREATE TABLE branches(name VARCHAR(255), head VARCHAR(255), parent VARCHAR(255), lock_domain_id VARCHAR(64));
         CREATE UNIQUE INDEX branch_name on branches(name);
        ";

    execute_sql(sql_connection, sql)
        .await
        .context("error creating branch table and index")
}

pub async fn create_workspace_branch_table(sql_connection: &mut sqlx::AnyConnection) -> Result<()> {
    let sql = "CREATE TABLE current_branch(name VARCHAR(255), commit_id VARCHAR(255));
        ";

    execute_sql(sql_connection, sql)
        .await
        .context("error creating current branch table")
}

pub async fn insert_current_branch(
    connection: &mut sqlx::AnyConnection,
    branch_name: &str,
    commit_id: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO current_branch VALUES(?, ?);")
        .bind(branch_name)
        .bind(commit_id)
        .execute(connection)
        .await
        .context("error inserting current branch")?;

    Ok(())
}

pub async fn update_current_branch(
    connection: &mut sqlx::AnyConnection,
    branch_name: &str,
    commit_id: &str,
) -> Result<()> {
    sqlx::query("UPDATE current_branch SET name=?, commit_id=?;")
        .bind(branch_name)
        .bind(commit_id)
        .execute(connection)
        .await
        .context("error updating current branch")?;

    Ok(())
}

pub async fn read_current_branch(connection: &mut sqlx::AnyConnection) -> Result<(String, String)> {
    let row = sqlx::query(
        "SELECT name, commit_id 
             FROM current_branch;",
    )
    .fetch_one(connection)
    .await
    .context("error fetching current branch")?;

    let name = row.get("name");
    let commit_id = row.get("commit_id");

    Ok((name, commit_id))
}

#[span_fn]
pub async fn create_branch_command(name: &str) -> Result<()> {
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

#[span_fn]
pub async fn list_branches_command() -> Result<()> {
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
