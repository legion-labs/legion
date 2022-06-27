mod data_lake;
mod errors;
mod provider;

pub use self::provider::IngestionProvider;
pub use data_lake::{DataLakeConnection, DataLakeProvider};
pub use errors::{Error, Result};

use crate::api::ingestion::{
    server::{
        InsertBlockRequest, InsertBlockResponse, InsertProcessRequest, InsertProcessResponse,
        InsertStreamRequest, InsertStreamResponse,
    },
    Api,
};
use async_trait::async_trait;
use lgn_telemetry::decode_block_and_payload;
use std::sync::Arc;

pub struct Server {
    pub provider: Arc<dyn IngestionProvider + Send + Sync>,
}

impl Server {
    pub fn new(provider: Arc<dyn IngestionProvider + Send + Sync>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Api for Server {
    async fn insert_block(
        &self,
        request: InsertBlockRequest,
    ) -> lgn_online::server::Result<InsertBlockResponse> {
        let (block, payload) = decode_block_and_payload(request.body.as_ref())?;
        self.provider.insert_block(block, payload).await?;
        Ok(InsertBlockResponse::Status200 {})
    }

    async fn insert_process(
        &self,
        request: InsertProcessRequest,
    ) -> lgn_online::server::Result<InsertProcessResponse> {
        self.provider.insert_process(request.body.into()).await?;
        Ok(InsertProcessResponse::Status200 {})
    }

    async fn insert_stream(
        &self,
        request: InsertStreamRequest,
    ) -> lgn_online::server::Result<InsertStreamResponse> {
        self.provider.insert_stream(request.body.into()).await?;
        Ok(InsertStreamResponse::Status200 {})
    }
}
