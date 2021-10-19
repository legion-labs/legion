use anyhow::Result;
use legion_analytics::prelude::*;
use std::path::Path;

pub async fn print_process_log(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
) -> Result<()> {
    for_each_process_log_entry(connection, data_path, process_id, |entry| {
        println!("{}", entry);
    })
    .await?;
    Ok(())
}

pub async fn print_logs_by_process(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
) -> Result<()> {
    for p in fetch_recent_processes(connection).await.unwrap() {
        println!("{} {} {}", p.start_time, p.process_id, p.exe);
        print_process_log(connection, data_path, &p.process_id).await?;
        println!();
    }
    Ok(())
}
