//! Source Control File System

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
