//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)

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

use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use instant::Duration;

use lgn_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_async::AsyncPlugin;
use lgn_config::Config;
use lgn_core::CorePlugin;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_grpc::{GRPCPlugin, GRPCPluginSettings};
use lgn_input::InputPlugin;
use lgn_renderer::RendererPlugin;
use lgn_streamer::StreamerPlugin;
use lgn_telemetry::prelude::*;
use lgn_telemetry_sink::TelemetryGuard;
use lgn_transform::prelude::*;

#[cfg(feature = "standalone")]
mod standalone;
#[cfg(feature = "standalone")]
use standalone::build_standalone;

#[derive(Parser, Debug)]
#[clap(name = "Legion Labs runtime engine")]
#[clap(
    about = "Server that will run with runtime data, and execute world simulation, ready to be streamed to a runtime client.",
    version,
    author
)]
struct Args {
    /// The address to listen on
    #[clap(long)]
    addr: Option<String>,
    /// Path to folder containing the project index
    #[clap(long)]
    project: Option<String>,
    /// Path to folder containing the content storage files
    #[clap(long)]
    cas: Option<String>,
    /// Path to the game manifest
    #[clap(long)]
    manifest: Option<String>,
    /// Root object to load, usually a world
    #[clap(long)]
    root: Option<String>,
    #[clap(long)]
    egui: bool,

    /// If supplied, starts with a window display, and collects input locally
    #[cfg(feature = "standalone")]
    #[clap(long)]
    standalone: bool,
}

#[allow(clippy::too_many_lines)]
fn main() {
    let _telemetry_guard = TelemetryGuard::new().unwrap();
    let _telemetry_thread_guard = TelemetryThreadGuard::new();

    trace_scope!();

    let args = Args::parse();
    let settings = Config::new();

    let server_addr = {
        let url = args
            .addr
            .as_deref()
            .unwrap_or_else(|| settings.get_or("runtime_srv.server_addr", "[::1]:50052"));
        url.parse::<SocketAddr>()
            .unwrap_or_else(|err| panic!("Invalid server_addr '{}': {}", url, err))
    };

    let project_dir = {
        if let Some(params) = args.project {
            PathBuf::from(params)
        } else {
            settings
                .get_absolute_path("runtime_srv.project_dir")
                .unwrap_or_else(|| PathBuf::from("test/sample-data"))
        }
    };

    let content_store_addr = args
        .cas
        .map_or_else(|| project_dir.join("temp"), PathBuf::from);

    let game_manifest = args.manifest.map_or_else(
        || project_dir.join("runtime").join("game.manifest"),
        PathBuf::from,
    );

    let mut assets_to_load = Vec::<ResourceTypeAndId>::new();

    // default root object is in sample data
    // /world/sample_1.ent

    let root_asset = args
        .root
        .as_deref()
        .unwrap_or("(1d9ddd99aad89045,af7e6ef0-c271-565b-c27a-b8cd93c3546a)"); // should map to the runtime_entity generated by
                                                                               // (1c0ff9e497b0740f,45d44b64-b97c-486d-a0d7-151974c28263)|1d9ddd99aad89045 --
                                                                               // check output.index
    if let Ok(asset_id) = root_asset.parse::<ResourceTypeAndId>() {
        assets_to_load.push(asset_id);
    }

    #[cfg(feature = "standalone")]
    let standalone = args.standalone;

    #[cfg(not(feature = "standalone"))]
    let standalone = false;

    let mut app = App::new();

    app
        // Start app with 60 fps
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(CorePlugin::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(TransformPlugin::default())
        .insert_resource(AssetRegistrySettings::new(
            content_store_addr,
            game_manifest,
            assets_to_load,
        ))
        .add_plugin(AssetRegistryPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(RendererPlugin::new(standalone, args.egui, true));

    #[cfg(feature = "standalone")]
    if standalone {
        build_standalone(&mut app);
    }

    if !standalone {
        app.add_plugin(AsyncPlugin::default())
            .insert_resource(GRPCPluginSettings::new(server_addr))
            .add_plugin(GRPCPlugin::default())
            .add_plugin(StreamerPlugin::default());
    }

    app.run();
}
