use telemetry::telemetry_ingestion_proto::{
    telemetry_ingestion_server::TelemetryIngestion,
    telemetry_ingestion_server::TelemetryIngestionServer, InsertProcessRequest, InsertReply,
};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct LocalIngestionService {}

#[tonic::async_trait]
impl TelemetryIngestion for LocalIngestionService {
    async fn insert_process(
        &self,
        request: Request<InsertProcessRequest>,
    ) -> Result<Response<InsertReply>, Status> {
        dbg!(&request);

        let reply = InsertReply {
            msg: format!("Hello {}!", request.into_inner().id),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse()?;
    let service = LocalIngestionService::default();

    Server::builder()
        .add_service(TelemetryIngestionServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
