use anyhow::Result;
use telemetry::telemetry_ingestion_proto::{
    telemetry_ingestion_server::TelemetryIngestion, Block, InsertReply, Process, Stream,
};
use tonic::{Request, Response, Status};

pub struct LocalIngestionService {
    db_pool: sqlx::any::AnyPool,
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
        let process_info = request.into_inner();
        match self.db_pool.acquire().await {
            Ok(mut connection) => {
                let current_date: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
                if let Err(e) = sqlx::query("INSERT INTO processes VALUES(?,?,?,?,?,?,?,?,?,?);")
                    .bind(process_info.process_id.clone())
                    .bind(process_info.exe)
                    .bind(process_info.username)
                    .bind(process_info.realname)
                    .bind(process_info.computer)
                    .bind(process_info.distro)
                    .bind(process_info.cpu_brand)
                    .bind(process_info.tsc_frequency as i64)
                    .bind(process_info.start_time)
                    .bind(current_date.format("%Y-%m-%d").to_string())
                    .execute(&mut connection)
                    .await
                {
                    dbg!(&e);
                    return Err(Status::internal(format!(
                        "Error inserting into processes: {}",
                        e
                    )));
                }

                let reply = InsertReply {
                    msg: format!("OK {}", process_info.process_id),
                };

                Ok(Response::new(reply))
            }
            Err(e) => {
                return Err(Status::internal(format!("Error connecting to db: {}", e)));
            }
        }
    }

    async fn insert_stream(
        &self,
        request: Request<Stream>,
    ) -> Result<Response<InsertReply>, Status> {
        let reply = InsertReply {
            msg: format!("Hello {}!", request.into_inner().stream_id),
        };

        Ok(Response::new(reply))
    }

    async fn insert_block(&self, request: Request<Block>) -> Result<Response<InsertReply>, Status> {
        let reply = InsertReply {
            msg: format!("Hello {}!", request.into_inner().block_id),
        };

        Ok(Response::new(reply))
    }
}
