//! Telemetry Dump CLI
//!

// BEGIN - Legion Labs lints v0.5
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
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal
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
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow()]

mod process_log;
mod recent_processes;

use analytics::alloc_sql_pool;
use anyhow::{bail, Result};
use clap::{App, AppSettings, Arg, SubCommand};
use process_log::{print_logs_by_process, print_process_log};
use recent_processes::print_recent_processes;
use std::path::Path;
use telemetry::{init_thread_stream, log_str, LogLevel, TelemetrySystemGuard};

#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry_guard = TelemetrySystemGuard::new();
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
        .get_matches();

    let data_path = Path::new(matches.value_of("db").unwrap());
    let pool = alloc_sql_pool(data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    match matches.subcommand() {
        ("recent-processes", Some(_command_match)) => {
            print_recent_processes(&mut connection).await;
        }
        ("logs-by-process", Some(_command_match)) => {
            print_logs_by_process(&mut connection, data_path).await?;
        }
        ("process-log", Some(command_match)) => {
            let process_id = command_match.value_of("process-id").unwrap();
            print_process_log(&mut connection, data_path, process_id).await?;
        }
        (command_name, _args) => {
            log_str(LogLevel::Info, "unknown subcommand match");
            bail!("unknown subcommand match: {:?}", &command_name);
        }
    }
    Ok(())
}
