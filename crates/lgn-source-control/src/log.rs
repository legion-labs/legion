use anyhow::{Context, Result};
use lgn_tracing::span_fn;

use crate::{find_branch_commits, Workspace};

#[span_fn]
pub async fn log_command() -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;
    let (branch_name, current_commit) = workspace.backend.get_current_branch().await?;

    println!(
        "This workspace is on branch {} at commit {}",
        &branch_name, &current_commit
    );

    let repo_branch = workspace.index_backend.read_branch(&branch_name).await?;

    let commits = find_branch_commits(&workspace, &repo_branch)
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
