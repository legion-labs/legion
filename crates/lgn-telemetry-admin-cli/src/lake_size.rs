use anyhow::Result;
use lgn_blob_storage::BlobStorage;
use lgn_tracing::prelude::*;
use sqlx::Executor;
use sqlx::Row;
use std::sync::Arc;

#[allow(clippy::cast_possible_wrap)]
pub async fn fill_block_sizes(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
) -> Result<()> {
    let rows = connection
        .fetch_all(
            "SELECT blocks.block_id, length(payloads.payload) as in_db_size
             FROM blocks
             LEFT JOIN payloads ON blocks.block_id = payloads.block_id
             WHERE payload_size IS NULL",
        )
        .await?;

    let mut nb = 0;
    for r in rows {
        let block_id: String = r.try_get("block_id")?;
        let mut blob_size: Option<i64> = r.try_get("in_db_size")?;
        if blob_size.is_none() {
            if let Some(stats) = blob_storage.get_blob_info(&block_id).await? {
                blob_size = Some(stats.size as i64);
            } else {
                error!("blob not found for block {}", block_id);
            }
        }
        if blob_size.is_some() {
            let size = blob_size.unwrap();
            println!("{} {}", block_id, size);
            sqlx::query("UPDATE blocks set payload_size = ? WHERE block_id = ?")
                .bind(size)
                .bind(block_id)
                .execute(&mut *connection)
                .await?;
        }
        nb += 1;
    }
    println!("{} rows", nb);
    Ok(())
}
