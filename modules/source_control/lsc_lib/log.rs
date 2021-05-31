use crate::*;
use chrono::{DateTime, Local};

pub fn log_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let work_branch = read_current_branch(&workspace_root)?;
    match find_branch_commits_command() {
        Ok(commits) => {
            for c in commits {
                let utc = DateTime::parse_from_rfc3339(&c.date_time_utc)
                    .expect("Error reading commit date");
                let local_time: DateTime<Local> = DateTime::from(utc);
                let branch_id;
                if c.id == work_branch.head {
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
