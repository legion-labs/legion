//! Legion App
//!
//! This crate is about everything concerning the highest-level, application
//! layer of a Legion app.

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
#![allow(clippy::struct_excessive_bools)]

mod bench;
mod build;
mod cargo;
mod changed_since;
mod check;
mod clippy;
mod config;
mod context;
mod doc;
mod error;
mod fix;
mod fmt;
mod git;
mod installer;
mod lint;
mod term;
mod test;
mod tools;
mod utils;

//mod sources;
//mod hash;
//mod package;

use clap::{Parser, Subcommand};
use lgn_telemetry::TelemetryThreadGuard;
use lgn_telemetry_sink::TelemetryGuard;

use error::Error;

/// A convenience type alias to return `Error`s from functions.
pub type Result<T> = std::result::Result<T, Error>;

/// Legion CLI
#[derive(Parser)]
#[clap(name = "lgn-monorepo")]
#[clap(about = "Legion Monorepo CLI")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // Cargo commands:
    /// Run `cargo bench`
    #[clap(name = "bench")]
    Bench(bench::Args),
    /// Run `cargo build`
    // the argument must be Boxed due to it's size and clippy (it's quite large by comparison to others.)
    #[clap(name = "build")]
    Build(build::Args),
    /// Run `cargo check`
    #[clap(name = "check")]
    Check(check::Args),
    /// Run `cargo clippy`
    #[clap(name = "clippy")]
    Clippy(clippy::Args),
    /// Run `cargo doc`
    #[clap(name = "doc")]
    Doc(doc::Args),
    /// Only list the packages with changes since the specified Git reference
    /// Run `cargo fix`
    #[clap(name = "fix")]
    Fix(fix::Args),
    /// Run `cargo fmt`
    #[clap(name = "fmt")]
    Fmt(fmt::Args),
    /// Run `cargo tests`
    #[clap(name = "test")]
    Test(test::Args),

    // Non Cargo commands:
    /// List packages changed since merge base with the given commit
    ///
    /// Note that this compares against the merge base (common ancestor) of the specified commit.
    /// For example, if origin/master is specified, the current working directory will be compared
    /// against the point at which it branched off of origin/master.
    #[clap(name = "changed-since")]
    ChangedSince(changed_since::Args),
    /// Run tools installation
    #[clap(name = "tools")]
    Tools(tools::Args),
    /// Run tools installation
    #[clap(name = "lint")]
    Lint(lint::Args),
}

fn main() -> Result<()> {
    let _telemetry_guard = TelemetryGuard::new().unwrap();
    let _telemetry_thread_guard = TelemetryThreadGuard::new();

    let args = Cli::parse();
    let ctx = context::Context::new()?;

    match args.command {
        Commands::Build(args) => build::run(&args, &ctx)?,
        Commands::Bench(args) => bench::run(args, &ctx)?,
        Commands::Check(args) => check::run(&args, &ctx)?,
        Commands::Clippy(args) => clippy::run(&args, &ctx)?,
        Commands::Doc(args) => doc::run(args, &ctx)?,
        Commands::Fix(args) => fix::run(args, &ctx)?,
        Commands::Fmt(args) => fmt::run(args, &ctx)?,
        Commands::Test(args) => test::run(args, &ctx)?,

        Commands::ChangedSince(args) => changed_since::run(&args, &ctx)?,
        Commands::Lint(args) => lint::run(&args, &ctx)?,
        Commands::Tools(args) => tools::run(&args, &ctx)?,
    };
    Ok(())
}
