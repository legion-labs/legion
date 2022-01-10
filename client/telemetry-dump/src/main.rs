//! Telemetry Dump CLI

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
//#![]

mod process_log;
mod process_metrics;
mod process_search;
mod process_thread_events;

use std::path::PathBuf;

use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use lgn_analytics::alloc_sql_pool;
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

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry_guard = TelemetryGuard::default().unwrap();

    let args = Cli::parse();

    let data_path = args.db;
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
            print_logs_by_process(&mut connection, &data_path).await?;
        }
        Commands::ProcessLog { process_id } => {
            print_process_log(&mut connection, &data_path, &process_id).await?;
        }
        Commands::ProcessThreadEvents { process_id } => {
            print_process_thread_events(&mut connection, &data_path, &process_id).await?;
        }
        Commands::PrintChromeTrace { process_id } => {
            print_chrome_trace(&pool, &data_path, &process_id).await?;
        }
        Commands::ProcessMetrics { process_id } => {
            print_process_metrics(&mut connection, &data_path, &process_id).await?;
        }
    }
    Ok(())
}
