use anyhow::Result;
use lgn_tracing::span_fn;

use crate::Workspace;

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
