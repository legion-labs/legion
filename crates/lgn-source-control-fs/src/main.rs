//! Source Control File System

// crate-specific lint exceptions:
#![allow(clippy::exit, clippy::too_many_lines, clippy::wildcard_imports)]

use std::path::PathBuf;

use clap::{AppSettings, Parser};
use lgn_source_control_fs::run;
use lgn_telemetry_sink::{Config, TelemetryGuard};
use lgn_tracing::*;

/// Legion Source Control
#[derive(Parser, Debug)]
#[clap(name = "Legion Source Control File System")]
#[clap(
    about = "A fuse implementation of the Legion Source Control",
    version,
    author
)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    #[clap(name = "debug", short, long, help = "Enable debug logging")]
    debug: bool,

    #[clap(name = "index_url", help = "The LSC index URL")]
    index_url: String,

    #[clap(name = "mountpoint", help = "The filesystem mount point")]
    mountpoint: PathBuf,

    #[clap(name = "branch", default_value = "main", help = "The branch to mount")]
    branch: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let _telemetry_guard = if args.debug {
        TelemetryGuard::default()
            .unwrap()
            .with_log_level(LevelFilter::Debug)
    } else {
        TelemetryGuard::new(Config::default(), false)
            .unwrap()
            .with_log_level(LevelFilter::Info)
    };

    span_scope!("lgn_source_control_fs::main");

    let index_backend = lgn_source_control::new_index_backend(&args.index_url)?;

    tokio::select! {
        r = lgn_cli_utils::wait_for_termination() => r,
        r = run(index_backend, args.branch, args.mountpoint) => r,
    }
}
