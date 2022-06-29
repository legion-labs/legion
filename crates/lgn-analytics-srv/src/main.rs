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

mod cache;
mod call_tree;
mod cumulative_call_graph;
mod cumulative_call_graph_handler;
mod cumulative_call_graph_node;
mod lakehouse;
mod log_entry;
mod metrics;
mod scope;
mod thread_block_processor;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use lakehouse::provider::DataLakeConnection;
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::prelude::*;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[clap(name = "Legion Performance Analytics Server")]
#[clap(about = "Legion Performance Analytics Server", version, author)]
#[clap(arg_required_else_help(true))]
struct Cli {
    #[clap(long, default_value = "[::1]:9090")]
    listen_endpoint: SocketAddr,

    #[clap(subcommand)]
    spec: DataLakeSpec,

    #[clap(help = "optional local path or s3://bucket/root")]
    lakehouse_uri: Option<String>,
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

#[tokio::main]
#[span_fn]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry_guard = TelemetryGuardBuilder::default()
        .with_ctrlc_handling()
        .build();
    info!("starting analytics server");
    span_scope!("analytics-srv::main");
    let args = Cli::parse();
    let connection = match args.spec {
        DataLakeSpec::Local {
            data_lake_path,
            cache_path,
        } => DataLakeConnection::new_local(data_lake_path, cache_path, args.lakehouse_uri).await?,
        DataLakeSpec::Remote {
            db_uri,
            s3_lake_url,
            s3_cache_url,
        } => {
            DataLakeConnection::new_remote(&db_uri, &s3_lake_url, s3_cache_url, args.lakehouse_uri)
                .await?
        }
    };

    Ok(())
}
