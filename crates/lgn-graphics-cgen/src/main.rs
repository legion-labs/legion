//! Renderer plugin.

// crate-specific lint exceptions:

use std::path::PathBuf;

use anyhow::Result;
use clap::{AppSettings, Parser};
use lgn_graphics_cgen::run::{run, CGenBuildResult, CGenContextBuilder};
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::LevelFilter;

#[derive(Parser, Debug)]
#[clap(name = "graphics-cgen")]
#[clap(about = "Graphics code generator", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    /// Verbose mode
    #[clap(long, short)]
    verbose: bool,
    /// Sets the input file to use
    #[clap(long, short)]
    crate_name: String,
    /// Sets the input file to use
    #[clap(long, short)]
    input: PathBuf,
    /// Sets the output folder for code generation
    #[clap(long, short)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let res = main_internal();

    match &res {
        Ok(result) => {
            println!("Input dependencies:");
            for dep in &result.input_dependencies {
                println!("{}", dep.display());
            }
        }
        Err(err) => {
            for i in err.chain() {
                eprintln!("{}", i);
            }
        }
    }
    res.map(|_| ())
}

fn main_internal() -> Result<CGenBuildResult> {
    // read command line arguments
    let args = Cli::parse();

    let log_level = if args.verbose {
        LevelFilter::Trace
    } else {
        LevelFilter::Warn
    };

    let _telemety_guard = TelemetryGuard::default().unwrap().with_log_level(log_level);

    // initialize context
    let mut ctx_builder = CGenContextBuilder::new();
    ctx_builder.set_root_file(&args.input)?;
    ctx_builder.set_out_dir(&args.output)?;
    ctx_builder.set_crate_name(&args.crate_name);

    // run the generation
    run(&ctx_builder.build())
}
