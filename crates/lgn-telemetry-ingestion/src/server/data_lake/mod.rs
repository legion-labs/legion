mod local;
mod remote;
mod sql;

use async_trait::async_trait;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry::types::{Block, BlockPayload, Process, Stream};
use lgn_tracing::{async_span_scope, info};
use std::sync::Arc;

use super::provider::IngestionProvider;
use super::Result;

#[derive(Clone)]
pub struct DataLakeConnection {
    pub db_pool: sqlx::any::AnyPool,
    pub blob_storage: Arc<dyn BlobStorage>,
}

impl DataLakeConnection {
    pub fn new(db_pool: sqlx::AnyPool, blob_storage: Arc<dyn BlobStorage>) -> Self {
        Self {
            db_pool,
            blob_storage,
        }
    }
}

pub struct DataLakeProvider {
    connection: DataLakeConnection,
}

impl DataLakeProvider {
    pub fn new(connection: DataLakeConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl IngestionProvider for DataLakeProvider {
    async fn insert_block(&self, block: Block, payload: BlockPayload) -> Result<()> {
        async_span_scope!("DataLake::insert_block");
        info!("new block {}", block.block_id);

        let mut connection = self.connection.db_pool.acquire().await?;

        let encoded_payload = payload.encode();
        let payload_size = encoded_payload.len();
        if payload_size >= 128 * 1024 {
            self.connection
                .blob_storage
                .write_blob(&block.block_id, &encoded_payload)
                .await?;
        } else {
            sqlx::query("INSERT INTO payloads values(?,?);")
                .bind(block.block_id.clone())
                .bind(encoded_payload)
                .execute(&mut connection)
                .await?;
        }

        #[allow(clippy::cast_possible_wrap)]
        sqlx::query("INSERT INTO blocks VALUES(?,?,?,?,?,?,?,?);")
            .bind(block.block_id.clone())
            .bind(block.stream_id)
            .bind(block.begin_time)
            .bind(block.begin_ticks as i64)
            .bind(block.end_time)
            .bind(block.end_ticks as i64)
            .bind(block.nb_objects)
            .bind(payload_size as i64)
            .execute(&mut connection)
            .await?;
        Ok(())
    }

    async fn insert_process(&self, process: Process) -> Result<()> {
        async_span_scope!("DataLake::insert_process");
        info!("new process [{}] {}", process.exe, process.process_id);

        let mut connection = self.connection.db_pool.acquire().await?;

        let current_date: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        #[allow(clippy::cast_possible_wrap)]
        sqlx::query("INSERT INTO processes VALUES(?,?,?,?,?,?,?,?,?,?,?,?);")
            .bind(process.process_id.clone())
            .bind(process.exe)
            .bind(process.username)
            .bind(process.realname)
            .bind(process.computer)
            .bind(process.distro)
            .bind(process.cpu_brand)
            .bind(process.tsc_frequency as i64)
            .bind(process.start_time)
            .bind(process.start_ticks as i64)
            .bind(current_date.format("%Y-%m-%d").to_string())
            .bind(process.parent_process_id.clone())
            .execute(&mut connection)
            .await?;
        Ok(())
    }

    async fn insert_stream(&self, stream: Stream) -> Result<()> {
        async_span_scope!("DataLake::insert_stream");

        let mut connection = self.connection.db_pool.acquire().await?;

        let dependencies_metadata = match stream.dependencies_metadata {
            Some(metadata) => metadata.encode(),
            None => Vec::new(),
        };
        let objects_metadata = match stream.objects_metadata {
            Some(metadata) => metadata.encode(),
            None => Vec::new(),
        };
        let tags = stream.tags.join(" ");
        let properties = serde_json::to_string(&stream.properties).unwrap();

        info!("new stream {} [{}]", stream.stream_id, tags);

        sqlx::query("INSERT INTO streams VALUES(?,?,?,?,?,?);")
            .bind(stream.stream_id.clone())
            .bind(stream.process_id)
            .bind(dependencies_metadata)
            .bind(objects_metadata)
            .bind(tags)
            .bind(properties)
            .execute(&mut connection)
            .await?;

        Ok(())
    }
}
