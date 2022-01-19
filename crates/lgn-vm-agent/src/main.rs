//! Manages legion engine processes lifetime within a VM
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)

// crate-specific lint exceptions:
//#![allow()]

mod config;

use std::process::Stdio;

use anyhow::Context;
use config::{CommandConfig, Config};
use lgn_cli_utils::termination_handler::AsyncTerminationHandler;
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::{debug, info};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::new().context("failed to read configuration")?;

    let _telemetry_guard = TelemetryGuard::default().unwrap();

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
