//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)
//!

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

use clap::Arg;
use instant::Duration;
use lgn_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_async::AsyncPlugin;
use lgn_core::CorePlugin;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_grpc::{GRPCPlugin, GRPCPluginSettings};
use lgn_input::InputPlugin;
use lgn_renderer::RendererPlugin;
use lgn_streamer::StreamerPlugin;
use lgn_telemetry::prelude::*;
use lgn_transform::prelude::*;

#[cfg(feature = "standalone")]
mod standalone;
use lgn_utils::Settings;
#[cfg(feature = "standalone")]
use standalone::build_standalone;

#[allow(clippy::too_many_lines)]
fn main() {
    lgn_logger::Logger::init(lgn_logger::Config::default()).unwrap();
    let _telemetry_guard = TelemetrySystemGuard::new();
    let _telemetry_thread_guard = TelemetryThreadGuard::new();
    trace_scope!();

    const ARG_NAME_ADDR: &str = "addr";
    const ARG_NAME_CAS: &str = "cas";
    const ARG_NAME_MANIFEST: &str = "manifest";
    const ARG_NAME_ROOT: &str = "root";
    const ARG_NAME_EGUI: &str = "egui";

    #[cfg(feature = "standalone")]
    const ARG_NAME_STANDALONE: &str = "standalone";

    let args = clap::App::new("Legion Labs runtime engine")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about("Server that will run with runtime data, and execute world simulation, ready to be streamed to a runtime client.")
        .arg(Arg::with_name(ARG_NAME_ADDR)
            .long(ARG_NAME_ADDR)
            .takes_value(true)
            .help("The address to listen on"))
        .arg(Arg::with_name(ARG_NAME_CAS)
            .long(ARG_NAME_CAS)
            .takes_value(true)
            .help("Path to folder containing the content storage files"))
        .arg(Arg::with_name(ARG_NAME_MANIFEST)
            .long(ARG_NAME_MANIFEST)
            .takes_value(true)
            .help("Path to the game manifest"))
        .arg(Arg::with_name(ARG_NAME_ROOT)
            .long(ARG_NAME_ROOT)
            .takes_value(true)
            .help("Root object to load, usually a world"))
        .arg(Arg::with_name(ARG_NAME_EGUI)
            .long(ARG_NAME_EGUI)
            .takes_value(false)
            .help("If supplied, starts with egui enabled"));

    #[cfg(feature = "standalone")]
    let args = args.arg(
        Arg::with_name(ARG_NAME_STANDALONE)
            .long(ARG_NAME_STANDALONE)
            .takes_value(false)
            .help("If supplied, starts with a window display, and collects input locally"),
    );

    let args = args.get_matches();

    let settings = Settings::new();

    let server_addr = {
        let url = args
            .value_of(ARG_NAME_ADDR)
            .unwrap_or_else(|| settings.get_or("runtime_srv.server_addr", "[::1]:50052"));
        url.parse::<SocketAddr>()
            .unwrap_or_else(|err| panic!("Invalid server_addr '{}': {}", url, err))
    };

    let content_store_addr = {
        if let Some(params) = args.value_of(ARG_NAME_CAS) {
            PathBuf::from(params)
        } else {
            settings
                .get_absolute_path("runtime_srv.cas")
                .unwrap_or_else(|| PathBuf::from("test/sample-data/temp"))
        }
    };

    let game_manifest = args
        .value_of(ARG_NAME_MANIFEST)
        .unwrap_or("test/sample-data/runtime/game.manifest");

    let mut assets_to_load = Vec::<ResourceTypeAndId>::new();

    // default root object is in sample data
    // /world/sample_1.ent

    let root_asset = args
        .value_of(ARG_NAME_ROOT)
        .unwrap_or("(aad89045,2c2d444d-0643-5628-a4a7-3980d3604fd0)");
    if let Ok(asset_id) = root_asset.parse::<ResourceTypeAndId>() {
        assets_to_load.push(asset_id);
    }

    #[cfg(feature = "standalone")]
    let standalone = args.is_present(ARG_NAME_STANDALONE);

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
            None,
        ))
        .add_plugin(AssetRegistryPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(RendererPlugin::new(
            standalone,
            args.is_present(ARG_NAME_EGUI),
        ));

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
