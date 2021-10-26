//! Telemetry Dump CLI
//!

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
mod process_thread_events;
mod recent_processes;

use crate::{
    process_metrics::print_process_metrics,
    process_thread_events::{print_chrome_trace, print_process_thread_events},
    recent_processes::{print_process_search, print_process_tree},
};
use anyhow::{bail, Result};
use clap::{App, AppSettings, Arg, SubCommand};
use legion_analytics::alloc_sql_pool;
use legion_telemetry::{init_thread_stream, log_str, LogLevel, TelemetrySystemGuard};
use process_log::{print_logs_by_process, print_process_log};
use recent_processes::print_recent_processes;
use std::path::Path;

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry_guard = TelemetrySystemGuard::new(None);
    init_thread_stream();
    let matches = App::new("Legion Telemetry Dump")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("CLI to query a local telemetry data lake")
        .arg(
            Arg::with_name("db")
                .required(true)
                .help("local path to folder containing telemetry.db3"),
        )
        .subcommand(
            SubCommand::with_name("recent-processes").about("prints a list of recent processes"),
        )
        .subcommand(
            SubCommand::with_name("find-processes")
                .about("prints a list of recent processes matching the provided string")
                .arg(
                    Arg::with_name("filter")
                        .required(true)
                        .help("executable name filter"),
                ),
        )
        .subcommand(
            SubCommand::with_name("process-tree")
                .about("lists the process and its subprocesses")
                .arg(
                    Arg::with_name("process-id")
                        .required(true)
                        .help("process guid"),
                ),
        )
        .subcommand(
            SubCommand::with_name("logs-by-process").about("prints the logs of recent processes"),
        )
        .subcommand(
            SubCommand::with_name("process-log")
                .about("prints the log streams of the process")
                .arg(
                    Arg::with_name("process-id")
                        .required(true)
                        .help("process guid"),
                ),
        )
        .subcommand(
            SubCommand::with_name("process-thread-events")
                .about("prints the thread streams of the process")
                .arg(
                    Arg::with_name("process-id")
                        .required(true)
                        .help("process guid"),
                ),
        )
        .subcommand(
            SubCommand::with_name("print-chrome-trace")
                .about("outputs a file compatible with chrome://tracing/")
                .arg(
                    Arg::with_name("process-id")
                        .required(true)
                        .help("process guid"),
                ),
        )
        .subcommand(
            SubCommand::with_name("process-metrics")
                .about("prints the metrics streams of the process")
                .arg(
                    Arg::with_name("process-id")
                        .required(true)
                        .help("process guid"),
                ),
        )
        .get_matches();

    let data_path = Path::new(matches.value_of("db").unwrap());
    let pool = alloc_sql_pool(data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    match matches.subcommand() {
        ("recent-processes", Some(_command_match)) => {
            print_recent_processes(&mut connection).await;
        }
        ("find-processes", Some(command_match)) => {
            let filter = command_match.value_of("filter").unwrap();
            print_process_search(&mut connection, filter).await;
        }
        ("process-tree", Some(command_match)) => {
            let process_id = command_match.value_of("process-id").unwrap();
            print_process_tree(&mut connection, process_id).await;
        }
        ("logs-by-process", Some(_command_match)) => {
            print_logs_by_process(&mut connection, data_path).await?;
        }
        ("process-log", Some(command_match)) => {
            let process_id = command_match.value_of("process-id").unwrap();
            print_process_log(&mut connection, data_path, process_id).await?;
        }
        ("process-thread-events", Some(command_match)) => {
            let process_id = command_match.value_of("process-id").unwrap();
            print_process_thread_events(&mut connection, data_path, process_id).await?;
        }
        ("print-chrome-trace", Some(command_match)) => {
            let process_id = command_match.value_of("process-id").unwrap();
            print_chrome_trace(&mut connection, data_path, process_id).await?;
        }
        ("process-metrics", Some(command_match)) => {
            let process_id = command_match.value_of("process-id").unwrap();
            print_process_metrics(&mut connection, data_path, process_id).await?;
        }
        (command_name, _args) => {
            log_str(LogLevel::Info, "unknown subcommand match");
            bail!("unknown subcommand match: {:?}", &command_name);
        }
    }
    Ok(())
}
