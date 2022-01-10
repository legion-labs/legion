use std::io::Write;
use std::{fs::OpenOptions, path::PathBuf};

use anyhow::Result;
use lgn_telemetry_proto::ingestion::telemetry_ingestion_server::TelemetryIngestion;
use lgn_telemetry_proto::ingestion::InsertReply;
use lgn_telemetry_proto::telemetry::{Block, Process, Stream};
use lgn_tracing::{error, info};
use prost::Message;
use tonic::{Request, Response, Status};

pub struct LocalIngestionService {
    db_pool: sqlx::any::AnyPool,
    blocks_dir: PathBuf,
}

impl LocalIngestionService {
    pub fn new(db_pool: sqlx::AnyPool, blocks_dir: PathBuf) -> Self {
        Self {
            db_pool,
            blocks_dir,
        }
    }
}

#[tonic::async_trait]
impl TelemetryIngestion for LocalIngestionService {
    async fn insert_process(
        &self,
        request: Request<Process>,
    ) -> Result<Response<InsertReply>, Status> {
        let process_info = request.into_inner();
        info!(
            "new process [{}] {}",
            process_info.exe, process_info.process_id
        );
        match self.db_pool.acquire().await {
            Ok(mut connection) => {
                let current_date: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
                #[allow(clippy::cast_possible_wrap)]
                if let Err(e) =
                    sqlx::query("INSERT INTO processes VALUES(?,?,?,?,?,?,?,?,?,?,?,?);")
                        .bind(process_info.process_id.clone())
                        .bind(process_info.exe)
                        .bind(process_info.username)
                        .bind(process_info.realname)
                        .bind(process_info.computer)
                        .bind(process_info.distro)
                        .bind(process_info.cpu_brand)
                        .bind(process_info.tsc_frequency as i64)
                        .bind(process_info.start_time)
                        .bind(process_info.start_ticks as i64)
                        .bind(current_date.format("%Y-%m-%d").to_string())
                        .bind(process_info.parent_process_id.clone())
                        .execute(&mut connection)
                        .await
                {
                    error!("{}", &e);
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
        let stream_info = request.into_inner();
        match self.db_pool.acquire().await {
            Ok(mut connection) => {
                let dependencies_metadata = match stream_info.dependencies_metadata {
                    Some(metadata) => metadata.encode_to_vec(),
                    None => Vec::new(),
                };
                let objects_metadata = match stream_info.objects_metadata {
                    Some(metadata) => metadata.encode_to_vec(),
                    None => Vec::new(),
                };
                let tags = stream_info.tags.join(" ");
                let properties = serde_json::to_string(&stream_info.properties).unwrap();
                info!("new stream [{}] {}", tags, stream_info.stream_id);
                if let Err(e) = sqlx::query("INSERT INTO streams VALUES(?,?,?,?,?,?);")
                    .bind(stream_info.stream_id.clone())
                    .bind(stream_info.process_id)
                    .bind(dependencies_metadata)
                    .bind(objects_metadata)
                    .bind(tags)
                    .bind(properties)
                    .execute(&mut connection)
                    .await
                {
                    error!("{}", &e);
                    return Err(Status::internal(format!(
                        "Error inserting into streams: {}",
                        e
                    )));
                }

                let reply = InsertReply {
                    msg: format!("OK {}", stream_info.stream_id),
                };
                Ok(Response::new(reply))
            }
            Err(e) => {
                return Err(Status::internal(format!("Error connecting to db: {}", e)));
            }
        }
    }

    async fn insert_block(&self, request: Request<Block>) -> Result<Response<InsertReply>, Status> {
        let block = request.into_inner();
        info!("new block {}", block.block_id);
        let payload = match block.payload {
            Some(p) => p,
            None => {
                return Err(Status::internal(String::from("Payload not found in block")));
            }
        };

        let mut connection = match self.db_pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                return Err(Status::internal(format!("Error connecting to db: {}", e)));
            }
        };

        let encoded_payload = payload.encode_to_vec();
        if encoded_payload.len() >= 128 * 1024 {
            let block_path = self.blocks_dir.join(&block.block_id);
            //todo: use async-aware file I/O
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&block_path)
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(&encoded_payload) {
                        return Err(Status::internal(format!("Error writing block file: {}", e)));
                    }
                }
                Err(e) => {
                    return Err(Status::internal(format!(
                        "Error creating block file: {}",
                        e
                    )));
                }
            }
        } else if let Err(e) = sqlx::query("INSERT INTO payloads values(?,?);")
            .bind(block.block_id.clone())
            .bind(encoded_payload)
            .execute(&mut connection)
            .await
        {
            error!("{}", &e);
            return Err(Status::internal(format!(
                "Error inserting into payloads: {}",
                e
            )));
        }

        #[allow(clippy::cast_possible_wrap)]
        if let Err(e) = sqlx::query("INSERT INTO blocks VALUES(?,?,?,?,?,?,?);")
            .bind(block.block_id.clone())
            .bind(block.stream_id)
            .bind(block.begin_time)
            .bind(block.begin_ticks as i64)
            .bind(block.end_time)
            .bind(block.end_ticks as i64)
            .bind(block.nb_objects)
            .execute(&mut connection)
            .await
        {
            error!("{}", &e);
            return Err(Status::internal(format!(
                "Error inserting into blocks: {}",
                e
            )));
        }
        let reply = InsertReply {
            msg: format!("OK {}", block.block_id),
        };
        Ok(Response::new(reply))
    }
}
