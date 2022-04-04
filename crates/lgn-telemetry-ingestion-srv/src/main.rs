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

mod grpc_ingestion_service;
mod local_data_lake;
mod remote_data_lake;
mod sql_migration;
mod sql_telemetry_db;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use lgn_telemetry_proto::ingestion::telemetry_ingestion_server::TelemetryIngestionServer;
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::prelude::*;
use local_data_lake::connect_to_local_data_lake;
use remote_data_lake::connect_to_remote_data_lake;
use std::net::SocketAddr;
use tonic::transport::Server;
use warp::Filter;

#[derive(Parser, Debug)]
#[clap(name = "Legion Telemetry Ingestion Server")]
#[clap(about = "Legion Telemetry Ingestion Server", version, author)]
#[clap(arg_required_else_help(true))]
struct Cli {
    #[clap(long, default_value = "[::1]:8080")]
    listen_endpoint: SocketAddr, //grpc

    #[clap(long, default_value = "[::1]:8081")]
    listen_endpoint_http: SocketAddr,

    #[clap(subcommand)]
    spec: DataLakeSpec,
}

#[derive(Subcommand, Debug)]
enum DataLakeSpec {
    Local { path: PathBuf },
    Remote { db_uri: String, s3_url: String },
}

async fn serve_grpc(args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let service = match &args.spec {
        DataLakeSpec::Local { path } => connect_to_local_data_lake(path.clone()).await?,
        DataLakeSpec::Remote { db_uri, s3_url } => {
            connect_to_remote_data_lake(db_uri, s3_url).await?
        }
    };
    Server::builder()
        .add_service(TelemetryIngestionServer::new(service))
        .serve(args.listen_endpoint)
        .await?;
    Ok(())
}

async fn insert_process_request(
    body: serde_json::value::Value,
) -> Result<warp::reply::Response, warp::Rejection> {
    info!("insert_process {}", body);
    Ok(http::response::Response::builder()
        .status(500)
        .body(hyper::body::Body::from("allô"))
        .unwrap())
}

async fn serve_http(args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let command_filter = warp::path("telemetryingestion")
        .and(warp::path("insertprocess"))
        // .and(warp::body::bytes())
        .and(warp::body::json())
        .and_then(insert_process_request);
    warp::serve(command_filter)
        .run(args.listen_endpoint_http)
        .await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry_guard = TelemetryGuardBuilder::default()
        .with_ctrlc_handling()
        .build();
    let args = Cli::parse();
    tokio::select! {
        _ = serve_grpc(&args) => {
        },
        _ = serve_http(&args) => {
        }
    }
    Ok(())
}
