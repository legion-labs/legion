//! Perf report generation
//!

mod edition_latency;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use lgn_analytics::prelude::*;
use lgn_blob_storage::LocalBlobStorage;

/// Legion Editor Performance Report
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(arg_required_else_help(true))]
struct Cli {
    /// local path to folder containing telemetry.db3
    db: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compute editor latency
    #[clap(name = "edition-latency")]
    EditorLatency {
        /// The process guid
        process_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let data_path = Path::new(&args.db);
    let blocks_folder = data_path.join("blobs");
    let blob_storage = Arc::new(LocalBlobStorage::new(blocks_folder).await?);
    let pool = alloc_sql_pool(data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    match args.command {
        Commands::EditorLatency { process_id } => {
            edition_latency::print_edition_latency(&mut connection, blob_storage, &process_id)
                .await?;
        }
    }

    Ok(())
}
