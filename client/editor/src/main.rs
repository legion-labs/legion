//! Edutir client executable

// BEGIN - Legion Labs lints v0.4
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
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
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
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
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// END - Legion Labs standard lints v0.4
// crate-specific exceptions:
#![allow()]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use async_std::future::timeout;
use clap::Arg;
use legion_app::prelude::*;
use legion_async::AsyncPlugin;
use legion_editor_proto::{editor_client::*, InitializeStreamRequest};
use legion_tauri::{legion_tauri_command, TauriPluginSettings};
use std::{error::Error, time::Duration};

struct Config {
    server_addr: String,
}

impl Config {
    fn new(args: &clap::ArgMatches<'_>) -> anyhow::Result<Self> {
        Ok(Self {
            server_addr: args
                .value_of("server-addr")
                .unwrap_or("http://[::1]:50051")
                .parse()?,
        })
    }

    fn new_from_environment() -> anyhow::Result<Self> {
        let args = clap::App::new("Legion Labs editor")
            .author(clap::crate_authors!())
            .version(clap::crate_version!())
            .about("Legion Labs editor.")
            .arg(
                Arg::with_name("server-addr")
                    .long("server-addr")
                    .takes_value(true)
                    .help("The address of the editor server to connect to"),
            )
            .get_matches();

        Self::new(&args)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new_from_environment()?;
    let builder = tauri::Builder::default()
        .manage(config)
        .invoke_handler(tauri::generate_handler![initialize_stream]);

    let (tauri_settings, tauri_plugin) =
        TauriPluginSettings::new_with_plugin(builder, tauri::generate_context!());

    App::new()
        .insert_non_send_resource(tauri_settings)
        .add_plugin(tauri_plugin)
        .add_plugin(AsyncPlugin {})
        .run();

    Ok(())
}

#[legion_tauri_command]
async fn initialize_stream(
    config: tauri::State<'_, Config>,
    rtc_session_description: String,
) -> anyhow::Result<String> {
    let mut client = timeout(
        Duration::from_secs(3),
        EditorClient::connect(config.server_addr.clone()),
    )
    .await??;

    let rtc_session_description = base64::decode(rtc_session_description)?;
    let request = tonic::Request::new(InitializeStreamRequest {
        rtc_session_description,
    });

    let response = client.initialize_stream(request).await?.into_inner();

    if response.error.is_empty() {
        Ok(base64::encode(response.rtc_session_description))
    } else {
        Err(anyhow::format_err!("{}", response.error))
    }
}
