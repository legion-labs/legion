use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Arg;
use legion_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings, DataBuildSettings};
use legion_async::AsyncPlugin;
use legion_data_runtime::ResourceId;
use legion_grpc::{GRPCPlugin, GRPCPluginSettings};
use legion_resource_registry::{ResourceRegistryPlugin, ResourceRegistrySettings};
use legion_streamer::StreamerPlugin;
use legion_telemetry::prelude::*;
use legion_transform::TransformPlugin;

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
                .help("Path to the build index"),
        )
        .arg(
            Arg::with_name(ARG_NAME_DATABUILD_CLI)
                .long(ARG_NAME_DATABUILD_CLI)
                .takes_value(true)
                .help("Path to data build command line interface"),
        )
        .get_matches();

    let addr = args
        .value_of(ARG_NAME_ADDR)
        .unwrap_or("[::1]:50051")
        .parse()
        .unwrap();

    let project_folder = args
        .value_of(ARG_NAME_PROJECT)
        .unwrap_or("test/sample-data");

    let content_store_addr = args
        .value_of(ARG_NAME_CAS)
        .unwrap_or("test/sample-data/temp");

    let game_manifest = args
        .value_of(ARG_NAME_MANIFEST)
        .unwrap_or("test/sample-data/runtime/game.manifest");

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
        let buildindex = args.value_of(ARG_NAME_BUILDINDEX).map_or_else(
            || Path::new(content_store_addr).join("build.index"),
            PathBuf::from,
        );

        Some(DataBuildSettings::new(build_bin, buildindex))
    };

    let assets_to_load: Vec<ResourceId> = Vec::new();

    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(AsyncPlugin {})
        .insert_resource(ResourceRegistrySettings::new(project_folder))
        .add_plugin(ResourceRegistryPlugin::default())
        .insert_resource(GRPCPluginSettings::new(addr))
        .add_plugin(GRPCPlugin {})
        .add_plugin(StreamerPlugin {})
        .add_plugin(EditorPlugin {})
        .add_plugin(TransformPlugin::default())
        .insert_resource(AssetRegistrySettings::new(
            content_store_addr,
            game_manifest,
            assets_to_load,
            databuild_settings,
        ))
        .add_plugin(AssetRegistryPlugin::default())
        .run();
}
