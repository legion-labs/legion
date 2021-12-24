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

mod cargo;
mod changed_since;
mod clippy;
mod config;
mod context;
mod error;
mod git;
//mod hash;
mod list;
//mod package;
mod term;
mod utils;

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
    /// Clones repos
    #[clap(name = "clippy")]
    Clippy(clippy::Args),

    /// Only list the packages with changes since the specified Git reference
    #[clap(name = "list")]
    List(list::Args),

    /// List packages changed since merge base with the given commit
    ///
    /// Note that this compares against the merge base (common ancestor) of the specified commit.
    /// For example, if origin/master is specified, the current working directory will be compared
    /// against the point at which it branched off of origin/master.
    #[clap(name = "changed-since")]
    ChangedSince(changed_since::Args),
}

fn main() -> Result<()> {
    let _telemetry_guard = TelemetryGuard::new().unwrap();
    let _telemetry_thread_guard = TelemetryThreadGuard::new();

    let args = Cli::parse();
    let context = context::Context::new()?;

    match &args.command {
        Commands::Clippy(args) => clippy::run(args, &context)?,
        Commands::List(args) => list::run(args, &context)?,
        Commands::ChangedSince(args) => changed_since::run(args, &context)?,
    };
    Ok(())
}
