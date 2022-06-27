//! The Governance server executable.

use clap::{Parser, Subcommand};
use lgn_governance::Config;
// use lgn_telemetry_sink::TelemetryGuardBuilder;
// use lgn_tracing::{async_span_scope, LevelFilter};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,

    #[clap(short, long)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize the stack.
    ///
    /// Must be called exactly once after a fresh install.
    #[clap(name = "init-stack", about = "Initialize the stack")]
    InitStack {
        #[clap(help = "The initialization key, as specified on the server's command line")]
        init_key: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    // let _telemetry_guard = TelemetryGuardBuilder::default()
    //     .with_local_sink_max_level(if args.debug {
    //         LevelFilter::Debug
    //     } else {
    //         LevelFilter::Info
    //     })
    //     .build();

    // async_span_scope!("lgc::main");

    let config = Config::load()?;
    let client = config.instantiate_client().await?;

    match args.command {
        Commands::InitStack { init_key } => client.init_stack(&init_key).await?,
    }

    Ok(())
}
