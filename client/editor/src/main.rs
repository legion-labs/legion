//! Editor client executable

// BEGIN - Legion Labs lints v0.5
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
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow()]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod interop;

use config::Config;
use interop::js::editor::{JSResourceDescription, JSSearchResourcesResponse};

use legion_app::prelude::*;
use legion_async::AsyncPlugin;
use legion_editor_proto::{editor_client::EditorClient, SearchResourcesRequest};
use legion_grpc::client::Client as GRPCClient;
use legion_streaming_proto::{streamer_client::StreamerClient, InitializeStreamRequest};
use legion_tauri::{legion_tauri_command, TauriPlugin, TauriPluginSettings};
use legion_telemetry::prelude::*;
use std::{error::Error, str::FromStr};
use tauri::async_runtime::Mutex;
use tonic::codegen::http::Uri;

fn main() -> Result<(), Box<dyn Error>> {
    let _telemetry_guard = TelemetrySystemGuard::new(None);
    let _telemetry_thread_guard = TelemetryThreadGuard::new();
    let config = Config::new_from_environment()?;
    let grpc_client = GRPCClient::new(Uri::from_str(&config.server_addr)?);
    let streamer_client = Mutex::new(StreamerClient::new(grpc_client.clone()));
    let editor_client = Mutex::new(EditorClient::new(grpc_client));
    let builder = tauri::Builder::default()
        .manage(config)
        .manage(streamer_client)
        .manage(editor_client)
        .invoke_handler(tauri::generate_handler![
            initialize_stream,
            search_resources
        ]);

    App::new()
        .insert_non_send_resource(TauriPluginSettings::new(builder))
        .add_plugin(TauriPlugin::new(tauri::generate_context!()))
        .add_plugin(AsyncPlugin {})
        .run();
    Ok(())
}

#[legion_tauri_command]
async fn initialize_stream(
    streamer_client: tauri::State<'_, Mutex<StreamerClient<GRPCClient>>>,
    rtc_session_description: String,
) -> anyhow::Result<String> {
    let rtc_session_description = base64::decode(rtc_session_description)?;
    let request = tonic::Request::new(InitializeStreamRequest {
        rtc_session_description,
    });

    let mut streamer_client = streamer_client.lock().await;

    let response = streamer_client
        .initialize_stream(request)
        .await?
        .into_inner();

    if response.error.is_empty() {
        Ok(base64::encode(response.rtc_session_description))
    } else {
        Err(anyhow::format_err!("{}", response.error))
    }
}

#[legion_tauri_command]
async fn search_resources(
    editor_client: tauri::State<'_, Mutex<EditorClient<GRPCClient>>>,
) -> anyhow::Result<JSSearchResourcesResponse> {
    let mut editor_client = editor_client.lock().await;

    let mut result = JSSearchResourcesResponse::default();

    let mut search_token = String::new();

    loop {
        let request = tonic::Request::new(SearchResourcesRequest { search_token });

        let response = editor_client.search_resources(request).await?.into_inner();

        search_token = response.next_search_token;
        result.resource_descriptions.extend(
            response
                .resource_descriptions
                .into_iter()
                .map(|x| x.into())
                .collect::<Vec<JSResourceDescription>>(),
        );

        if search_token.is_empty() {
            return Ok(result);
        }
    }
}
