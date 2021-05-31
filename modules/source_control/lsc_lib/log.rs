use crate::*;
use chrono::{DateTime, Local};

pub fn log_command() -> Result<(), String> {
    match find_branch_commits_command() {
        Ok(commits) => {
            for c in commits {
                let utc = DateTime::parse_from_rfc3339(&c.date_time_utc)
                    .expect("Error reading commit date");
                let local_time: DateTime<Local> = DateTime::from(utc);
                println!(
                    "{} {} {} {}",
                    c.id,
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
