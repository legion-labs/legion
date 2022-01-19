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

use std::path::PathBuf;

use analytics_service::AnalyticsService;
use anyhow::{Context, Result};
use lgn_analytics::alloc_sql_pool;
use lgn_telemetry_proto::analytics::performance_analytics_server::PerformanceAnalyticsServer;
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::prelude::*;
use tonic::transport::Server;

fn get_data_directory() -> Result<PathBuf> {
    let folder =
        std::env::var("LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY").with_context(|| {
            String::from("Error reading env variable LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY")
        })?;
    Ok(PathBuf::from(folder))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry_guard = TelemetryGuard::default()
        .unwrap()
        .with_log_level(LevelFilter::Info)
        .with_ctrlc_handling();
    span_scope!("analytics-srv::main");
    let addr = "127.0.0.1:9090".parse()?;
    let data_dir = get_data_directory()?;
    let pool = alloc_sql_pool(&data_dir).await?;
    let service = AnalyticsService::new(pool, data_dir)
        .await
        .with_context(|| "allocating AnalyticsService")?;
    info!("service allocated");
    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(PerformanceAnalyticsServer::new(service)))
        .serve(addr)
        .await?;
    Ok(())
}
