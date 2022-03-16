//! Editor server executable
//!

use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;
use generic_data::plugin::GenericDataPlugin;
use grpc::TraceEventsReceiver;
use lgn_app::{prelude::*, AppExit, EventWriter, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_async::AsyncPlugin;
use lgn_config::Config;
use lgn_core::{CorePlugin, DefaultTaskPoolOptions};
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::Local;
use lgn_graphics_renderer::RendererPlugin;
use lgn_grpc::{GRPCPlugin, GRPCPluginSettings};
use lgn_input::InputPlugin;
use lgn_resource_registry::{ResourceRegistryPlugin, ResourceRegistrySettings};
use lgn_scripting::ScriptingPlugin;
use lgn_streamer::StreamerPlugin;
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{debug, warn};
use lgn_transform::TransformPlugin;
use sample_data::SampleDataPlugin;
use tokio::sync::mpsc;

mod grpc;
mod plugin;
mod property_inspector_plugin;
use lgn_window::WindowPlugin;
use property_inspector_plugin::PropertyInspectorPlugin;

mod resource_browser_plugin;
use resource_browser_plugin::{ResourceBrowserPlugin, ResourceBrowserSettings};

mod source_control_plugin;
use source_control_plugin::SourceControlPlugin;

mod channel_sink;
use channel_sink::ChannelSink;

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
    /// Enable a testing code path
    #[clap(long)]
    test: Option<String>,
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

    let source_control_path = settings.get_or("editor_srv.source_control", "../remote".to_string());

    let game_manifest_path = args.manifest.map_or_else(PathBuf::new, PathBuf::from);
    let assets_to_load = Vec::<ResourceTypeAndId>::new();

    let mut telemetry_config = lgn_telemetry_sink::Config::default();
    telemetry_config.enable_tokio_console_server = true;

    let (trace_events_sender, trace_events_receiver) = mpsc::unbounded_channel();

    let telemetry_guard = TelemetryGuardBuilder::new(telemetry_config)
        .add_sink(ChannelSink::new(trace_events_sender))
        .build()
        .expect("telemetry guard should be initialized once");

    let trace_events_receiver: TraceEventsReceiver = trace_events_receiver.into();

    let mut app = App::from_telemetry_guard(telemetry_guard);

    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
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
    .insert_resource(ResourceRegistrySettings::new(
        project_folder,
        source_control_path,
    ))
    .add_plugin(ResourceRegistryPlugin::default())
    .insert_resource(GRPCPluginSettings::new(server_addr))
    .insert_resource(trace_events_receiver)
    .add_plugin(GRPCPlugin::default())
    .add_plugin(InputPlugin::default())
    .add_plugin(RendererPlugin::default())
    .add_plugin(StreamerPlugin::default())
    .add_plugin(EditorPlugin::default())
    .insert_resource(ResourceBrowserSettings::new(default_scene))
    .add_plugin(ResourceBrowserPlugin::default())
    .add_plugin(PropertyInspectorPlugin::default())
    .add_plugin(SourceControlPlugin::default())
    .add_plugin(TransformPlugin::default())
    .add_plugin(GenericDataPlugin::default())
    .add_plugin(ScriptingPlugin::default())
    .add_plugin(SampleDataPlugin::default())
    .add_plugin(lgn_graphics_data::GraphicsPlugin::default())
    .add_plugin(WindowPlugin {
        add_primary_window: false,
        exit_on_close: false,
    });

    if let Some(test_name) = args.test {
        match test_name.as_str() {
            "lifecycle" => {
                app.add_system(lifecycle_test);
            }
            _ => panic!("Unknown test '{}'", test_name),
        }
    }

    app.run();
}

fn lifecycle_test(
    mut app_exit_events: EventWriter<'_, '_, AppExit>,
    mut frame_counter: Local<'_, u32>,
) {
    *frame_counter += 1;
    warn!("Frame {}", *frame_counter);
    if *frame_counter == 10 {
        debug!("Exiting");
        app_exit_events.send(AppExit);
    }
}
