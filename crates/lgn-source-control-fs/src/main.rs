//! Source Control File System

// crate-specific lint exceptions:
#![allow(clippy::exit, clippy::wildcard_imports)]

use std::path::PathBuf;

use clap::Parser;
use lgn_source_control::RepositoryName;
use lgn_source_control_fs::run;
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::*;

/// Legion Source Control
#[derive(Parser, Debug)]
#[clap(name = "Legion Source Control File System")]
#[clap(
    about = "A fuse implementation of the Legion Source Control",
    version,
    author
)]
#[clap(arg_required_else_help(true))]
struct Cli {
    #[clap(name = "debug", short, long, help = "Enable debug logging")]
    debug: bool,

    #[clap(name = "index_url", help = "The LSC index URL")]
    index_url: String,

    #[clap(name = "mountpoint", help = "The filesystem mount point")]
    mountpoint: PathBuf,

    #[clap(
        name = "repository-name",
        default_value = "default",
        help = "The repository name"
    )]
    repository_name: RepositoryName,

    #[clap(name = "branch", default_value = "main", help = "The branch to mount")]
    branch: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let _telemetry_guard = if args.debug {
        TelemetryGuardBuilder::default()
            .with_local_sink_max_level(LevelFilter::Debug)
            .build()
    } else {
        TelemetryGuardBuilder::default()
            .with_local_sink_enabled(false)
            .build()
    };

    span_scope!("lgn_source_control_fs::main");

    let repository_index =
        lgn_source_control::Config::load_and_instantiate_repository_index().await?;

    let index = repository_index
        .load_repository(&args.repository_name)
        .await?;

    tokio::select! {
        r = lgn_cli_utils::wait_for_termination() => r,
        r = run(index, args.branch, args.mountpoint) => r,
    }
}
