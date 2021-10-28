//! Editor client executable

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
#![allow()]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod interop;

use config::Config;
use interop::js::editor::{
    IntoVec, JSGetResourcePropertiesRequest, JSGetResourcePropertiesResponse,
    JSSearchResourcesResponse, JSUpdateResourcePropertiesRequest,
    JSUpdateResourcePropertiesResponse,
};

use legion_app::prelude::*;
use legion_async::AsyncPlugin;
use legion_auth::authenticator::Authenticator;
use legion_editor_proto::{
    editor_client::EditorClient, GetResourcePropertiesRequest, SearchResourcesRequest,
    UpdateResourcePropertiesRequest,
};
use legion_grpc::client::Client as GRPCClient;
use legion_streaming_proto::{streamer_client::StreamerClient, InitializeStreamRequest};
use legion_tauri::{legion_tauri_command, TauriPlugin, TauriPluginSettings};
use legion_telemetry::prelude::*;
use simple_logger::SimpleLogger;
use std::error::Error;
use tauri::async_runtime::Mutex;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new_from_environment()?;

    let _telemetry_guard = TelemetrySystemGuard::new(Some(Box::new(
        SimpleLogger::new().with_level(config.log_level),
    )));
    let _telemetry_thread_guard = TelemetryThreadGuard::new();

    let authenticator = Authenticator::from_authorization_url(&config.authorization_url)?;
    let grpc_client = GRPCClient::new(config.server_addr.clone());
    let streamer_client = Mutex::new(StreamerClient::new(grpc_client.clone()));
    let editor_client = Mutex::new(EditorClient::new(grpc_client));

    let builder = tauri::Builder::default()
        .manage(config)
        .manage(authenticator)
        .manage(streamer_client)
        .manage(editor_client)
        .invoke_handler(tauri::generate_handler![
            authenticate,
            initialize_stream,
            search_resources,
            get_resource_properties,
            update_resource_properties,
            on_receive_control_message,
            on_send_edition_command,
            on_video_close,
            on_video_chunk_received,
        ]);

    App::new()
        .insert_non_send_resource(TauriPluginSettings::new(builder))
        .add_plugin(TauriPlugin::new(tauri::generate_context!()))
        .add_plugin(AsyncPlugin {})
        .run();
    Ok(())
}

#[tauri::command]
fn on_send_edition_command(json_command: &str) {
    log::info!("sending edition_command={}", json_command);
}

#[tauri::command]
fn on_receive_control_message(json_msg: &str) {
    log::info!("received control message. msg={}", json_msg);
}

#[tauri::command]
fn on_video_close() {
    flush_log_buffer();
    flush_metrics_buffer();
}

fn record_json_metric(desc: &'static MetricDesc, value: &json::JsonValue) {
    match value.as_i64() {
        Some(int_value) => {
            record_int_metric(desc, int_value as u64);
        }
        None => {
            log::error!("Error converting {} to i64", value);
        }
    }
}

#[tauri::command]
fn on_video_chunk_received(chunk_header: &str) {
    static CHUNK_INDEX_IN_FRAME_METRIC: MetricDesc = MetricDesc {
        name: "Chunk Index in Frame",
        unit: "",
    };

    static FRAME_ID_OF_CHUNK_RECEIVED: MetricDesc = MetricDesc {
        name: "Frame ID of chunk received",
        unit: "",
    };

    match json::parse(chunk_header) {
        Ok(header) => {
            record_json_metric(
                &CHUNK_INDEX_IN_FRAME_METRIC,
                &header["chunk_index_in_frame"],
            );
            record_json_metric(&FRAME_ID_OF_CHUNK_RECEIVED, &header["frame_id"]);
        }
        Err(e) => {
            log::error!("Error parsing chunk header: {}", e);
        }
    }
}

#[legion_tauri_command]
async fn authenticate(authenticator: tauri::State<'_, Authenticator>) -> anyhow::Result<String> {
    authenticator.get_authorization_code().await
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

    // TODO: perhaps this method is too smart and should be more straighforward, and let the
    // pagination be done at the Javascript level.
    let mut search_token = String::new();

    loop {
        let request = tonic::Request::new(SearchResourcesRequest { search_token });

        let response = editor_client.search_resources(request).await?.into_inner();

        search_token = response.next_search_token;
        result
            .resource_descriptions
            .extend(response.resource_descriptions.into_vec());

        if search_token.is_empty() {
            return Ok(result);
        }
    }
}

#[legion_tauri_command]
async fn get_resource_properties(
    editor_client: tauri::State<'_, Mutex<EditorClient<GRPCClient>>>,
    request: JSGetResourcePropertiesRequest,
) -> anyhow::Result<JSGetResourcePropertiesResponse> {
    let mut editor_client = editor_client.lock().await;

    let request: GetResourcePropertiesRequest = request.into();
    let request = tonic::Request::new(request);

    Ok(editor_client
        .get_resource_properties(request)
        .await?
        .into_inner()
        .into())
}

#[legion_tauri_command]
async fn update_resource_properties(
    editor_client: tauri::State<'_, Mutex<EditorClient<GRPCClient>>>,
    request: JSUpdateResourcePropertiesRequest,
) -> anyhow::Result<JSUpdateResourcePropertiesResponse> {
    let mut editor_client = editor_client.lock().await;

    let request: UpdateResourcePropertiesRequest = request.into();
    let request = tonic::Request::new(request);

    Ok(editor_client
        .update_resource_properties(request)
        .await?
        .into_inner()
        .into())
}
