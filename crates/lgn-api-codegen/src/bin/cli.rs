//! Legion `OpenApi` code generator CLI.
//!
//! Provides code generation commands.
//!

use clap::Parser;
use lgn_api_codegen::{generate, Language};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::LevelFilter;

#[derive(Parser, Debug)]
#[clap(name = "Legion API Code Generator")]
#[clap(
    about = "CLI to generate code based on an OpenAPI v3 specification.",
    version,
    author
)]
#[clap(arg_required_else_help(true))]
struct Args {
    #[clap(name = "debug", short, long, help = "Enable debug logging")]
    debug: bool,

    #[clap(
        arg_enum,
        name = "language",
        long,
        help = "The language to generate code for."
    )]
    language: Language,
    #[clap(
        name = "openapi-file",
        long,
        help = "The OpenAPI specification to use."
    )]
    openapi_file: String,
    #[clap(
        name = "output-dir",
        long,
        help = "The directory to output generated code to."
    )]
    output_dir: String,
}

#[allow(clippy::let_unit_value)]
fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let _telemetry_guard = TelemetryGuardBuilder::default()
        .with_local_sink_enabled(args.debug)
        .with_local_sink_max_level(LevelFilter::Debug)
        .build();

    generate(args.language, &args.openapi_file, &args.output_dir)?;

    Ok(())
}
