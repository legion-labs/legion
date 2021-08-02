use crate::*;
use chrono::{DateTime, Local};

pub fn log_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    let workspace_branch = read_current_branch(&workspace_root)?;
    println!(
        "This workspace is on branch {} at commit {}",
        &workspace_branch.name, &workspace_branch.head
    );

    let repo_branch =
        tokio_runtime.block_on(connection.query().read_branch(&workspace_branch.name))?;

    match tokio_runtime.block_on(find_branch_commits(&connection, &repo_branch)) {
        Ok(commits) => {
            for c in commits {
                let utc = DateTime::parse_from_rfc3339(&c.date_time_utc)
                    .expect("Error reading commit date");
                let local_time: DateTime<Local> = DateTime::from(utc);
                let branch_id;
                if c.id == workspace_branch.head {
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
