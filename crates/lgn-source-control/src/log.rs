use anyhow::{Context, Result};
use lgn_tracing::span_fn;

use crate::{
    connect_to_server, find_branch_commits, find_workspace_root, read_current_branch,
    read_workspace_spec, LocalWorkspaceConnection,
};

#[span_fn]
pub async fn log_command() -> Result<()> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let (branch_name, current_commit) = read_current_branch(workspace_connection.sql()).await?;
    println!(
        "This workspace is on branch {} at commit {}",
        &branch_name, &current_commit
    );

    let repo_branch = connection.index_backend().read_branch(&branch_name).await?;

    let commits = find_branch_commits(&connection, &repo_branch)
        .await
        .context("error fetching commits")?;

    for c in commits {
        let branch_id = if c.id == current_commit {
            format!("*{}", &c.id)
        } else {
            format!(" {}", &c.id)
        };

        println!(
            "{} {} {} {}",
            branch_id,
            c.timestamp.format("%Y-%m-%d %H:%M:%S"),
            c.owner,
            c.message
        );
    }

    Ok(())
}
