use anyhow::{ Result};
use tonic::{Request, Response, Status};
use telemetry::telemetry_ingestion_proto::{
    telemetry_ingestion_server::TelemetryIngestion, Block, InsertReply, Process, Stream,
};

pub struct LocalIngestionService {
    db_pool: sqlx::AnyPool,
}

impl LocalIngestionService {
    pub fn new(db_pool: sqlx::AnyPool) -> Self {
        Self { db_pool }
    }
}

#[tonic::async_trait]
impl TelemetryIngestion for LocalIngestionService {
    async fn insert_process(
        &self,
        request: Request<Process>,
    ) -> Result<Response<InsertReply>, Status> {
        dbg!(&request);
        dbg!(&self.db_pool);

        let reply = InsertReply {
            msg: format!("Hello {}!", request.into_inner().id),
        };

        Ok(Response::new(reply))
    }

    async fn insert_stream(
        &self,
        request: Request<Stream>,
    ) -> Result<Response<InsertReply>, Status> {
        dbg!(&request);

        let reply = InsertReply {
            msg: format!("Hello {}!", request.into_inner().stream_id),
        };

        Ok(Response::new(reply))
    }

    async fn insert_block(&self, request: Request<Block>) -> Result<Response<InsertReply>, Status> {
        dbg!(&request);

        let reply = InsertReply {
            msg: format!("Hello {}!", request.into_inner().block_id),
        };

        Ok(Response::new(reply))
    }
}
