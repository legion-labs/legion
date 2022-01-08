//! Renderer plugin.

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

    let _telemety_guard = TelemetryGuard::new().unwrap().with_log_level(log_level);

    // initialize context
    let mut ctx_builder = CGenContextBuilder::new();
    ctx_builder.set_root_file(&args.input)?;
    ctx_builder.set_outdir(&args.output)?;

    // run the generation
    run(&ctx_builder.build())
}
