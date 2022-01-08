//! Manages legion engine processes lifetime within a VM
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)

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
#![allow()]

mod config;

use std::process::Stdio;

use anyhow::Context;
use config::{CommandConfig, Config};
use lgn_cli::termination_handler::AsyncTerminationHandler;
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::{debug, info};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::new().context("failed to read configuration")?;

    let _telemetry_guard = TelemetryGuard::new().unwrap();

    debug!("Setting log level to {}.", config.log_level);
    info!("Root is set to: {}", config.root.to_string_lossy());

    let termination_handler = AsyncTerminationHandler::new()?;

    match config.command_config {
        CommandConfig::Run => run(termination_handler, &config).await,
    }
}

async fn run(termination_handler: AsyncTerminationHandler, config: &Config) -> anyhow::Result<()> {
    info!("Running VM-Agent...");

    let editor_server_bin_path = config.editor_server_bin_path();

    info!(
        "Using editor server at: {}",
        editor_server_bin_path.to_string_lossy(),
    );

    let mut process = Command::new(editor_server_bin_path)
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = process.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    loop {
        tokio::select! {
            _ = termination_handler.wait() => {
                info!("Ctrl+C signal caught: terminating.");
                return Ok(())
            }
            res = process.wait() => {
                res?;

                //return Ok(())
            },
            line = reader.next_line() => {
                if let Some(line) = line? {
                    println!("editor-srv: {}", line);
                }
            },
        }
    }
}
