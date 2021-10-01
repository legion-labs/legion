//! Telemetry Dump CLI
//!

// BEGIN - Legion Labs lints v0.3
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]
// END - Legion Labs standard lints v0.3
// crate-specific exceptions:
#![allow()]

use analytics::*;
use anyhow::*;
use clap::{App, AppSettings, Arg, SubCommand};
use std::path::Path;
use telemetry::*;

async fn print_recent_processes(connection: &mut sqlx::AnyConnection) {
    for p in fetch_recent_processes(connection).await.unwrap() {
        println!("{} {}", p.start_time, p.exe);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry_guard = TelemetrySystemGuard::new();
    init_thread_stream();
    let matches = App::new("Legion Telemetry Dump")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("CLI to query a local telemetry data lake")
        .arg(
            Arg::with_name("db")
                .required(true)
                .help("local path to folder containing telemetry.db3"),
        )
        .subcommand(
            SubCommand::with_name("recent-processes").about("prints a list of recent processes"),
        )
        .get_matches();

    let data_path = Path::new(matches.value_of("db").unwrap());
    let pool = alloc_sql_pool(data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    match matches.subcommand() {
        ("recent-processes", Some(_command_match)) => {
            print_recent_processes(&mut connection).await;
        }
        (command_name, _args) => {
            log_str(LogLevel::Info, "unknown subcommand match");
            bail!("unknown subcommand match: {:?}", &command_name);
        }
    }
    Ok(())
}
