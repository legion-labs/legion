//! Telemetry Ingestion Server
//!
//! Accepts telemetry data through grpc, stores the metadata in sqlite and the
//! raw event payload in local binary files.
//!
//! Env variables:
//!  - `LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY` : local directory where
//!    data will be dumped

// crate-specific lint exceptions:
//#![allow()]

mod ingestion_service;
mod local_data_lake;
mod local_telemetry_db;
mod remote_data_lake;

use std::path::PathBuf;

use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use lgn_telemetry_proto::ingestion::telemetry_ingestion_server::TelemetryIngestionServer;
use lgn_telemetry_sink::TelemetryGuard;
use local_data_lake::connect_to_local_data_lake;
use remote_data_lake::connect_to_remote_data_lake;
use std::net::SocketAddr;
use tonic::transport::Server;

#[derive(Parser, Debug)]
#[clap(name = "Legion Telemetry Ingestion Server")]
#[clap(about = "Legion Telemetry Ingestion Server", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    #[clap(long, default_value = "[::1]:8080")]
    listen_endpoint: SocketAddr,

    #[clap(subcommand)]
    spec: DataLakeSpec,
}

#[derive(Subcommand, Debug)]
enum DataLakeSpec {
    Local { path: PathBuf },
    Remote { db_uri: String, s3_url: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry_guard = TelemetryGuard::default().unwrap();
    let args = Cli::parse();
    let service = match args.spec {
        DataLakeSpec::Local { path } => connect_to_local_data_lake(path).await?,
        DataLakeSpec::Remote { db_uri, s3_url } => {
            connect_to_remote_data_lake(&db_uri, &s3_url).await?
        }
    };
    Server::builder()
        .add_service(TelemetryIngestionServer::new(service))
        .serve(args.listen_endpoint)
        .await?;
    Ok(())
}
