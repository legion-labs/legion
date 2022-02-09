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
mod auth;
mod cache;
mod call_tree;
mod call_tree_store;
mod cumulative_call_graph;
mod metrics;

use std::str::FromStr;
use std::{path::PathBuf, sync::Arc};

use analytics_service::AnalyticsService;
use anyhow::{Context, Result};
use auth::AuthLayer;
use clap::{AppSettings, Parser, Subcommand};
use lgn_blob_storage::{AwsS3BlobStorage, AwsS3Url, LocalBlobStorage, Lz4BlobStorageAdapter};
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
    Local {
        data_lake_path: PathBuf,
        cache_path: PathBuf,
    },
    Remote {
        db_uri: String,
        s3_lake_url: String,
        s3_cache_url: String,
    },
}

/// ``connect_to_local_data_lake`` serves a locally hosted data lake
///
/// # Errors
/// block storage must exist and sqlite database must accept connections
pub async fn connect_to_local_data_lake(
    data_lake_path: PathBuf,
    cache_path: PathBuf,
) -> Result<AnalyticsService> {
    let blocks_folder = data_lake_path.join("blobs");
    let data_lake_blobs = Arc::new(LocalBlobStorage::new(blocks_folder).await?);
    let cache_blobs = Arc::new(Lz4BlobStorageAdapter::new(
        LocalBlobStorage::new(cache_path).await?,
    ));
    let db_path = data_lake_path.join("telemetry.db3");
    let db_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(&db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    AnalyticsService::new(pool, data_lake_blobs, cache_blobs).await
}

/// ``connect_to_remote_data_lake`` serves a remote data lake through mysql and s3
///
/// # Errors
/// block storage must exist and mysql database must accept connections
pub async fn connect_to_remote_data_lake(
    db_uri: &str,
    s3_url_data_lake: &str,
    s3_url_cache: &str,
) -> Result<AnalyticsService> {
    info!("connecting to blob storage");
    let data_lake_blobs =
        Arc::new(AwsS3BlobStorage::new(AwsS3Url::from_str(s3_url_data_lake)?).await);
    let cache_blobs = Arc::new(Lz4BlobStorageAdapter::new(
        AwsS3BlobStorage::new(AwsS3Url::from_str(s3_url_cache)?).await,
    ));
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    AnalyticsService::new(pool, data_lake_blobs, cache_blobs).await
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
        DataLakeSpec::Local {
            data_lake_path,
            cache_path,
        } => connect_to_local_data_lake(data_lake_path, cache_path).await?,
        DataLakeSpec::Remote {
            db_uri,
            s3_lake_url,
            s3_cache_url,
        } => connect_to_remote_data_lake(&db_uri, &s3_lake_url, &s3_cache_url).await?,
    };

    let auth_layer = tower::ServiceBuilder::new()
        .layer(AuthLayer::default())
        .into_inner();

    let server = PerformanceAnalyticsServer::new(service);
    Server::builder()
        .accept_http1(true)
        .layer(auth_layer)
        .add_service(tonic_web::enable(server))
        .serve(args.listen_endpoint)
        .await?;
    Ok(())
}
