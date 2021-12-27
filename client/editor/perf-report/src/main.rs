//! Perf report generation
//!

mod edition_latency;
use std::path::Path;

use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use lgn_analytics::prelude::*;

/// Legion Editor Performance Report
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    /// local path to folder containing telemetry.db3
    #[clap(long)]
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
    let pool = alloc_sql_pool(data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    match args.command {
        Commands::EditorLatency { process_id } => {
            edition_latency::print_edition_latency(&mut connection, data_path, &process_id).await?;
        }
    }

    Ok(())
}
