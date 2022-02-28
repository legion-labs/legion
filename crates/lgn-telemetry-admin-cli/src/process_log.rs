use std::sync::Arc;

use anyhow::Result;
use lgn_analytics::prelude::*;
use lgn_blob_storage::BlobStorage;

pub async fn print_process_log(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    process_id: &str,
) -> Result<()> {
    for_each_process_log_entry(connection, blob_storage, process_id, |_time, entry| {
        println!("{}", entry);
    })
    .await?;
    Ok(())
}

pub async fn print_logs_by_process(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
) -> Result<()> {
    for p in fetch_recent_processes(connection).await.unwrap() {
        let process_info = p.process_info.unwrap();
        println!(
            "{} {} {}",
            process_info.start_time, process_info.process_id, process_info.exe
        );
        print_process_log(connection, blob_storage.clone(), &process_info.process_id).await?;
        println!();
    }
    Ok(())
}
