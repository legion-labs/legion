mod errors;

use errors::Result;
use http::{Request, Response};
use hyper::service::Service;
use lgn_telemetry::{
    api::components::{Process, Stream},
    encode_block_and_payload,
    types::{Block, BlockPayload},
};

/// A client for the ingestion service.
pub struct Client<Inner> {
    client: crate::api::ingestion::client::Client<Inner>,
    space_id: lgn_governance::types::SpaceId,
}

impl<Inner> Client<Inner> {
    /// Creates a new client.
    pub fn new(inner: Inner, base_uri: http::Uri) -> Self {
        Self {
            client: crate::api::ingestion::client::Client::new(inner, base_uri),
            space_id: "default".parse().unwrap(),
        }
    }
}

impl<Inner, ResBody> Client<Inner>
where
    Inner: Service<Request<hyper::Body>, Response = Response<ResBody>> + Send + Sync + Clone,
    Inner::Error: Into<lgn_online::client::Error>,
    Inner::Future: Send,
    ResBody: hyper::body::HttpBody + Send,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error,
{
    /// Insert a process into the ingestion server.
    ///
    /// # Errors
    ///
    /// This function will return an error if the call fails.
    pub async fn insert_process(&self, process: Process) -> Result<()> {
        use crate::api::ingestion::client::{InsertProcessRequest, InsertProcessResponse};

        match self
            .client
            .insert_process(InsertProcessRequest {
                space_id: self.space_id.clone().into(),
                body: process,
            })
            .await?
        {
            InsertProcessResponse::Status200 { .. } => Ok(()),
        }
    }

    /// Insert a stream into the ingestion server.
    ///
    /// # Errors
    ///
    /// This function will return an error if the call fails.
    pub async fn insert_stream(&self, stream: Stream) -> Result<()> {
        use crate::api::ingestion::client::{InsertStreamRequest, InsertStreamResponse};

        match self
            .client
            .insert_stream(InsertStreamRequest {
                space_id: self.space_id.clone().into(),
                body: stream,
            })
            .await?
        {
            InsertStreamResponse::Status200 { .. } => Ok(()),
        }
    }

    /// Insert a block into the ingestion server.
    ///
    /// # Errors
    ///
    /// This function will return an error if the call fails.
    pub async fn insert_block(&self, block: &Block, payload: BlockPayload) -> Result<()> {
        use crate::api::ingestion::client::{InsertBlockRequest, InsertBlockResponse};

        let bytes = encode_block_and_payload(block, payload)?;

        match self
            .client
            .insert_block(InsertBlockRequest {
                space_id: self.space_id.clone().into(),
                body: bytes.into(),
            })
            .await?
        {
            InsertBlockResponse::Status200 { .. } => Ok(()),
        }
    }
}
