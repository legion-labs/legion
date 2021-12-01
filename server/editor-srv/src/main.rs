use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::Arg;
use lgn_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings, DataBuildSettings};
use lgn_async::AsyncPlugin;
use lgn_core::CorePlugin;
use lgn_data_runtime::ResourceId;
use lgn_grpc::{GRPCPlugin, GRPCPluginSettings};
use lgn_renderer::RendererPlugin;
use lgn_input::InputPlugin;
use lgn_resource_registry::{ResourceRegistryPlugin, ResourceRegistrySettings};
use lgn_streamer::StreamerPlugin;
use lgn_telemetry::prelude::*;
use lgn_transform::TransformPlugin;
use lgn_utils::Settings;
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod grpc;
mod plugin;

use plugin::EditorPlugin;

fn main() {
    let _telemetry_guard = TelemetrySystemGuard::new(Some(Box::new(
        SimpleLogger::new().with_level(LevelFilter::Info),
    )));
    let _telemetry_thread_guard = TelemetryThreadGuard::new();
    trace_scope!();

    const ARG_NAME_ADDR: &str = "addr";
    const ARG_NAME_PROJECT: &str = "project";
    const ARG_NAME_CAS: &str = "cas";
    const ARG_NAME_MANIFEST: &str = "manifest";
    const ARG_NAME_BUILDINDEX: &str = "buildindex";
    const ARG_NAME_DATABUILD_CLI: &str = "databuild";

    let args = clap::App::new("Legion Labs editor server")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about("Editor server.")
        .arg(
            Arg::with_name(ARG_NAME_ADDR)
                .long(ARG_NAME_ADDR)
                .takes_value(true)
                .help("The address to listen on"),
        )
        .arg(
            Arg::with_name(ARG_NAME_PROJECT)
                .long(ARG_NAME_PROJECT)
                .takes_value(true)
                .help("Path to folder containing the project index"),
        )
        .arg(
            Arg::with_name(ARG_NAME_CAS)
                .long(ARG_NAME_CAS)
                .takes_value(true)
                .help("Path to folder containing the content storage files"),
        )
        .arg(
            Arg::with_name(ARG_NAME_MANIFEST)
                .long(ARG_NAME_MANIFEST)
                .takes_value(true)
                .help("Path to the game manifest"),
        )
        .arg(
            Arg::with_name(ARG_NAME_BUILDINDEX)
                .long(ARG_NAME_BUILDINDEX)
                .takes_value(true)
                .help("Path to the build index directory"),
        )
        .arg(
            Arg::with_name(ARG_NAME_DATABUILD_CLI)
                .long(ARG_NAME_DATABUILD_CLI)
                .takes_value(true)
                .help("Path to data build command line interface"),
        )
        .get_matches();

    let settings = Settings::new();

    let server_addr = {
        let url = args
            .value_of(ARG_NAME_ADDR)
            .unwrap_or_else(|| settings.get_or("editor_srv.server_addr", "[::1]:50051"));
        url.parse::<SocketAddr>()
            .unwrap_or_else(|err| panic!("Invalid server_addr '{}': {}", url, err))
    };

    let project_folder = {
        if let Some(params) = args.value_of(ARG_NAME_PROJECT) {
            PathBuf::from(params)
        } else {
            settings
                .get_absolute_path("editor_srv.project_dir")
                .unwrap_or_else(|| PathBuf::from("test/sample-data"))
        }
    };

    let content_store_path = args
        .value_of(ARG_NAME_CAS)
        .map_or_else(|| project_folder.join("temp"), PathBuf::from);

    let game_manifest_path = args.value_of(ARG_NAME_MANIFEST).map_or_else(
        || project_folder.join("runtime").join("game.manifest"),
        PathBuf::from,
    );

    let databuild_settings = {
        let build_bin = {
            args.value_of(ARG_NAME_DATABUILD_CLI).map_or_else(
                || {
                    std::env::current_exe().ok().map_or_else(
                        || panic!("cannot find test directory"),
                        |mut path| {
                            path.pop();
                            path.as_path().join("data-build.exe")
                        },
                    )
                },
                PathBuf::from,
            )
        };
        let buildindex = args
            .value_of(ARG_NAME_BUILDINDEX)
            .map_or_else(|| content_store_path.clone(), PathBuf::from);

        Some(DataBuildSettings::new(build_bin, buildindex))
    };

    let assets_to_load: Vec<ResourceId> = Vec::new();

    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(CorePlugin::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(AsyncPlugin::default())
        .insert_resource(AssetRegistrySettings::new(
            content_store_path,
            game_manifest_path,
            assets_to_load,
            databuild_settings,
        ))
        .add_plugin(AssetRegistryPlugin::default())
        .insert_resource(ResourceRegistrySettings::new(project_folder))
        .add_plugin(ResourceRegistryPlugin::default())
        .insert_resource(GRPCPluginSettings::new(server_addr))
        .add_plugin(GRPCPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(RendererPlugin::new(false))
        .add_plugin(StreamerPlugin::default())
        .add_plugin(EditorPlugin::default())
        .add_plugin(TransformPlugin::default())
        .run();
}
