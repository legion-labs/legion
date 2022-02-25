use anyhow::Result;
use lgn_blob_storage::BlobStorage;
use sqlx::Executor;
use sqlx::Row;
use std::sync::Arc;

pub async fn fill_block_sizes(
    connection: &mut sqlx::AnyConnection,
    _blob_storage: Arc<dyn BlobStorage>,
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
        let in_db_size: Option<i64> = r.try_get("in_db_size")?;
        println!("{} {}", block_id, in_db_size.unwrap_or(-1));
        nb += 1;
    }
    println!("{} rows", nb);
    Ok(())
}
