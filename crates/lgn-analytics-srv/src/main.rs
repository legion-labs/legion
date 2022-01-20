//! Legion Analytics Server
//!
//! Feeds data to the analytics-web interface.
//!
//! Env variables:
//!  - `LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY` : local telemetry
//!    directory
//!  - `LEGION_TELEMETRY_CACHE_DIRECTORY` : local directory where reusable
//!    computations will be stored

// crate-specific lint exceptions:
//#![allow()]

mod analytics_service;
mod cache;
mod call_tree;
mod call_tree_store;
mod cumulative_call_graph;
mod metrics;

use std::{path::PathBuf, sync::Arc};

use analytics_service::AnalyticsService;
use anyhow::{Context, Result};
use clap::{AppSettings, Parser, Subcommand};
use lgn_blob_storage::LocalBlobStorage;
use lgn_telemetry_proto::analytics::performance_analytics_server::PerformanceAnalyticsServer;
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::prelude::*;
use std::net::SocketAddr;
use tonic::transport::Server;

#[derive(Parser, Debug)]
#[clap(name = "Legion Performance Analytics Server")]
#[clap(about = "Legion Performance Analytics Server", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    #[clap(long, default_value = "[::1]:9090")]
    listen_endpoint: SocketAddr,

    #[clap(subcommand)]
    spec: DataLakeSpec,
}

#[derive(Subcommand, Debug)]
enum DataLakeSpec {
    Local { path: PathBuf },
    Remote { db_uri: String, s3_url: String },
}

/// ``connect_to_local_data_lake`` serves a locally hosted data lake
///
/// # Errors
/// block storage must exist and sqlite database must accept connections
pub async fn connect_to_local_data_lake(path: PathBuf) -> Result<AnalyticsService> {
    let blocks_folder = path.join("blobs");
    let blob_storage = Arc::new(LocalBlobStorage::new(blocks_folder).await?);
    let db_path = path.join("telemetry.db3");
    let db_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(&db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    AnalyticsService::new(pool, blob_storage).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry_guard = TelemetryGuard::default()
        .unwrap()
        .with_log_level(LevelFilter::Info)
        .with_ctrlc_handling();
    span_scope!("analytics-srv::main");
    let args = Cli::parse();
    let service = match args.spec {
        DataLakeSpec::Local { path } => connect_to_local_data_lake(path).await?,
        DataLakeSpec::Remote {
            db_uri: _,
            s3_url: _,
        } => {
            panic!("remote");
            // connect_to_remote_data_lake(&db_uri, &s3_url).await?
        }
    };

    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(PerformanceAnalyticsServer::new(service)))
        .serve(args.listen_endpoint)
        .await?;
    Ok(())
}
