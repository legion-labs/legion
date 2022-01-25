//! Telemetry Dump CLI

// crate-specific lint exceptions:
//#![]

mod process_log;
mod process_metrics;
mod process_search;
mod process_thread_events;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use lgn_analytics::alloc_sql_pool;
use lgn_blob_storage::LocalBlobStorage;
use lgn_telemetry_sink::TelemetryGuard;
use process_log::{print_logs_by_process, print_process_log};
use process_search::print_process_search;
use process_search::print_process_tree;
use process_search::print_recent_processes;

use crate::{
    process_metrics::print_process_metrics,
    process_thread_events::{print_chrome_trace, print_process_thread_events},
};

#[derive(Parser, Debug)]
#[clap(name = "Legion Telemetry Dump")]
#[clap(about = "CLI to query a local telemetry data lake", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    /// local path to folder containing telemetry.db3
    db: PathBuf,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Prints a list of recent processes
    #[clap(name = "recent-processes")]
    RecentProcesses,
    /// Prints a list of recent processes matching the provided string
    #[clap(name = "find-processes")]
    FindProcesses {
        /// executable name filter
        filter: String,
    },
    /// Lists the process and its subprocesses
    #[clap(name = "process-tree")]
    ProcessTree {
        /// process guid
        process_id: String,
    },
    /// Prints the logs of recent processes
    #[clap(name = "logs-by-process")]
    LogsByProcess,
    /// Prints the log streams of the process
    #[clap(name = "process-log")]
    ProcessLog {
        /// process guid
        process_id: String,
    },
    /// Prints the thread streams of the process
    #[clap(name = "process-thread-events")]
    ProcessThreadEvents {
        /// process guid
        process_id: String,
    },
    /// Outputs a file compatible with chrome://tracing/
    #[clap(name = "print-chrome-trace")]
    PrintChromeTrace {
        /// process guid
        process_id: String,
    },
    /// Prints the metrics streams of the process
    #[clap(name = "process-metrics")]
    ProcessMetrics {
        /// process guid
        process_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry_guard = TelemetryGuard::default().unwrap();

    let args = Cli::parse();

    let data_path = args.db;
    let blocks_folder = data_path.join("blobs");
    let blob_storage = Arc::new(LocalBlobStorage::new(blocks_folder).await?);
    let pool = alloc_sql_pool(&data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    match args.command {
        Commands::RecentProcesses => {
            print_recent_processes(&mut connection).await;
        }
        Commands::FindProcesses { filter } => {
            print_process_search(&mut connection, &filter).await;
        }
        Commands::ProcessTree { process_id } => {
            print_process_tree(&pool, &process_id).await?;
        }
        Commands::LogsByProcess => {
            print_logs_by_process(&mut connection, blob_storage).await?;
        }
        Commands::ProcessLog { process_id } => {
            print_process_log(&mut connection, blob_storage, &process_id).await?;
        }
        Commands::ProcessThreadEvents { process_id } => {
            print_process_thread_events(&mut connection, blob_storage, &process_id).await?;
        }
        Commands::PrintChromeTrace { process_id } => {
            print_chrome_trace(&pool, blob_storage, &process_id).await?;
        }
        Commands::ProcessMetrics { process_id } => {
            print_process_metrics(&mut connection, blob_storage, &process_id).await?;
        }
    }
    Ok(())
}
