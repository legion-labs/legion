use std::path::Path;

use anyhow::Result;
use legion_analytics::prelude::*;

pub async fn print_process_log(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
) -> Result<()> {
    for_each_process_log_entry(connection, data_path, process_id, |_time, entry| {
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
        let process_info = p.process_info.unwrap();
        println!(
            "{} {} {}",
            process_info.start_time, process_info.process_id, process_info.exe
        );
        print_process_log(connection, data_path, &process_info.process_id).await?;
        println!();
    }
    Ok(())
}
