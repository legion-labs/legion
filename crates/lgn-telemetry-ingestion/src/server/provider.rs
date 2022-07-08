use crate::server::Result;
use async_trait::async_trait;
use lgn_telemetry::types::{Block, BlockPayload, Process, Stream};

#[async_trait]
pub trait IngestionProvider {
    async fn insert_block(&self, block: Block, payload: BlockPayload) -> Result<()>;
    async fn insert_process(&self, process: Process) -> Result<()>;
    async fn insert_stream(&self, stream: Stream) -> Result<()>;
}
