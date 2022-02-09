//! Editor server executable
//!

use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;

use generic_data::plugin::GenericDataPlugin;
use lgn_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_async::AsyncPlugin;
use lgn_config::Config;
use lgn_core::{CorePlugin, DefaultTaskPoolOptions};
use lgn_data_runtime::ResourceTypeAndId;
use lgn_grpc::{GRPCPlugin, GRPCPluginSettings};
use lgn_input::InputPlugin;
use lgn_renderer::RendererPlugin;
use lgn_resource_registry::{ResourceRegistryPlugin, ResourceRegistrySettings};
use lgn_scripting::ScriptingPlugin;
use lgn_streamer::StreamerPlugin;
use lgn_transform::TransformPlugin;
use sample_data::SampleDataPlugin;

mod grpc;
mod plugin;
mod property_inspector_plugin;
use lgn_window::WindowPlugin;
use property_inspector_plugin::PropertyInspectorPlugin;

mod resource_browser_plugin;
use resource_browser_plugin::{ResourceBrowserPlugin, ResourceBrowserSettings};

#[cfg(test)]
#[path = "tests/test_resource_browser.rs"]
mod test_resource_browser;

#[cfg(test)]
#[path = "tests/test_property_inspector.rs"]
mod test_property_inspector;

use plugin::EditorPlugin;

#[derive(Parser, Debug)]
#[clap(name = "Legion Labs editor server")]
#[clap(about = "Editor server.", version, author)]
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
    /// Path to default scene (root asset) to load
    #[clap(long)]
    scene: Option<String>,
    /// Path to the game manifest
    #[clap(long)]
    manifest: Option<String>,
    #[clap(long)]
    egui: bool,
}

fn main() {
    let args = Args::parse();

    let settings = Config::new();

    let server_addr = {
        let url = args
            .addr
            .unwrap_or_else(|| settings.get_or("editor_srv.server_addr", "[::1]:50051".to_owned()));
        url.parse::<SocketAddr>()
            .unwrap_or_else(|err| panic!("Invalid server_addr '{}': {}", url, err))
    };

    let project_folder = {
        if let Some(params) = args.project {
            PathBuf::from(params)
        } else {
            settings
                .get_absolute_path("editor_srv.project_dir")
                .unwrap_or_else(|| PathBuf::from("tests/sample-data"))
        }
    };

    let content_store_path = args
        .cas
        .map_or_else(|| project_folder.join("temp"), PathBuf::from);

    std::mem::drop(std::fs::create_dir(&content_store_path));

    let default_scene = args
        .scene
        .unwrap_or_else(|| settings.get_or("editor_srv.default_scene", String::new()));

    let game_manifest_path = args.manifest.map_or_else(PathBuf::new, PathBuf::from);
    let assets_to_load = Vec::<ResourceTypeAndId>::new();

    let mut telemetry_config = lgn_telemetry_sink::Config::default();
    telemetry_config.enable_tokio_console_server = true;

    App::new(telemetry_config)
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(ScheduleRunnerPlugin::default())
        .insert_resource(DefaultTaskPoolOptions::new(1..=4))
        .add_plugin(CorePlugin::default())
        .add_plugin(AsyncPlugin::default())
        .insert_resource(AssetRegistrySettings::new(
            content_store_path,
            &game_manifest_path,
            assets_to_load,
        ))
        .add_plugin(AssetRegistryPlugin::default())
        .insert_resource(ResourceRegistrySettings::new(project_folder))
        .add_plugin(ResourceRegistryPlugin::default())
        .insert_resource(GRPCPluginSettings::new(server_addr))
        .add_plugin(GRPCPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(RendererPlugin::new(args.egui, false))
        .add_plugin(StreamerPlugin::default())
        .add_plugin(EditorPlugin::default())
        .insert_resource(ResourceBrowserSettings::new(default_scene))
        .add_plugin(ResourceBrowserPlugin::default())
        .add_plugin(PropertyInspectorPlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(GenericDataPlugin::default())
        .add_plugin(ScriptingPlugin::default())
        .add_plugin(SampleDataPlugin::default())
        .add_plugin(lgn_graphics_data::GraphicsPlugin::default())
        .add_plugin(WindowPlugin {
            add_primary_window: false,
            exit_on_close: false,
        })
        .run();
}
