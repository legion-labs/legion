use crate::*;
use chrono::{DateTime, Local};

pub async fn log_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let (branch_name, current_commit) = read_current_branch(workspace_connection.sql()).await?;
    println!(
        "This workspace is on branch {} at commit {}",
        &branch_name, &current_commit
    );

    let repo_branch = connection.query().read_branch(&branch_name).await?;

    match find_branch_commits(&connection, &repo_branch).await {
        Ok(commits) => {
            for c in commits {
                let utc = DateTime::parse_from_rfc3339(&c.date_time_utc)
                    .expect("Error reading commit date");
                let local_time: DateTime<Local> = DateTime::from(utc);
                let branch_id;
                if c.id == current_commit {
                    branch_id = format!("*{}", &c.id);
                } else {
                    branch_id = format!(" {}", &c.id);
                }
                println!(
                    "{} {} {} {}",
                    branch_id,
                    local_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    c.owner,
                    c.message
                );
            }
        }
        Err(e) => {
            return Err(format!("Error fetching commits: {}", e));
        }
    }
    Ok(())
}
