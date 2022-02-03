use anyhow::Result;
use lgn_tracing::span_fn;

use crate::{Branch, Workspace};

#[span_fn]
pub async fn create_branch_command(name: &str) -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;

    let (old_branch_name, old_workspace_commit) = workspace.backend.get_current_branch().await?;

    let old_repo_branch = workspace
        .index_backend
        .read_branch(&old_branch_name)
        .await?;
    let new_branch = Branch::new(
        String::from(name),
        old_workspace_commit.clone(),
        old_branch_name,
        old_repo_branch.lock_domain_id,
    );
    workspace.index_backend.insert_branch(&new_branch).await?;

    workspace
        .backend
        .set_current_branch(&new_branch.name, &new_branch.head)
        .await
        .map_err(Into::into)
}

#[span_fn]
pub async fn list_branches_command() -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;

    for branch in workspace.index_backend.read_branches().await? {
        println!(
            "{} head:{} parent:{}",
            branch.name, branch.head, branch.parent
        );
    }
    Ok(())
}
