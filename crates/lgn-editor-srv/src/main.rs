//! Editor server executable
//!

use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;
use generic_data::plugin::GenericDataPlugin;
use grpc::TraceEventsReceiver;
use lgn_app::{prelude::*, AppExit, EventWriter, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_async::AsyncPlugin;
use lgn_content_store::ContentStoreAddr;
use lgn_core::{CorePlugin, DefaultTaskPoolOptions};
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::Local;
use lgn_graphics_renderer::RendererPlugin;
use lgn_grpc::{GRPCPlugin, GRPCPluginSettings};
use lgn_hierarchy::HierarchyPlugin;
use lgn_input::InputPlugin;
use lgn_resource_registry::{
    settings::CompilationMode, ResourceRegistryPlugin, ResourceRegistrySettings,
};
use lgn_scene_plugin::ScenePlugin;
use lgn_scripting::ScriptingPlugin;
use lgn_source_control::RepositoryName;
use lgn_streamer::StreamerPlugin;
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{debug, info, warn, LevelFilter};
use lgn_transform::TransformPlugin;
use sample_data::SampleDataPlugin;
use serde::Deserialize;
use tokio::sync::broadcast;

mod grpc;
mod plugin;
mod property_inspector_plugin;
use lgn_window::WindowPlugin;
use property_inspector_plugin::PropertyInspectorPlugin;

mod resource_browser_plugin;
use resource_browser_plugin::{ResourceBrowserPlugin, ResourceBrowserSettings};

mod source_control_plugin;
use source_control_plugin::SourceControlPlugin;

mod broadcast_sink;
use broadcast_sink::BroadcastSink;

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
    listen_endpoint: Option<SocketAddr>,
    /// Path to folder containing the project index
    #[clap(long)]
    project_root: Option<PathBuf>,
    /// The name of the repository to load.
    #[clap(long, default_value = "default")]
    repository_name: RepositoryName,
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
    /// Build output database address.
    #[clap(long)]
    build_db: Option<String>,
    /// Enable hardware encoding.
    #[clap(long)]
    enable_hw_encoding: bool,
    /// The compilation mode of the editor.
    #[clap(long, default_value = "in-process")]
    compilers: CompilationMode,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    /// The endpoint to listen on.
    #[serde(default = "Config::default_listen_endpoint")]
    listen_endpoint: SocketAddr,

    /// The project root.
    #[serde(default = "Config::default_project_root")]
    project_root: PathBuf,

    /// The scene.
    #[serde(default)]
    scene: String,
}

impl Config {
    fn default_listen_endpoint() -> SocketAddr {
        "[::1]:50051".parse().unwrap()
    }

    fn default_project_root() -> PathBuf {
        PathBuf::from("tests/sample-data")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_endpoint: Self::default_listen_endpoint(),
            project_root: Self::default_project_root(),
            scene: "".to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let cwd = std::env::current_dir().unwrap();
    let config: Config = lgn_config::get("editor_server")
        .expect("failed to load config")
        .unwrap_or_default();

    let listen_endpoint = args.listen_endpoint.unwrap_or(config.listen_endpoint);

    info!("Listening on {}", listen_endpoint);

    let project_root = args.project_root.unwrap_or(config.project_root);
    let project_root = if project_root.is_absolute() {
        project_root
    } else {
        cwd.join(project_root)
    };

    info!("Project root: {}", project_root.display());

    let content_store_path = {
        let content_store_path = args.cas.map_or_else(
            || project_root.join("temp"),
            |path| {
                let path = PathBuf::from(path);
                if path.is_absolute() {
                    path
                } else {
                    cwd.join(path)
                }
            },
        );

        std::mem::drop(std::fs::create_dir(&content_store_path));
        ContentStoreAddr::from(content_store_path.to_str().unwrap())
    };

    let scene = args.scene.unwrap_or(config.scene);

    info!("Scene: {}", scene);

    let source_control_repository_index =
        lgn_source_control::Config::load_and_instantiate_repository_index()
            .expect("failed to load and instantiate a source control repository index");

    let repository_name = args.repository_name;

    info!("Repository name: {}", repository_name);

    let build_output_db_addr = {
        if let Some(build_db) = args.build_db {
            let cmd_line = if build_db.starts_with("mysql:") {
                build_db
            } else {
                let path = PathBuf::from(&build_db);
                if path.is_absolute() {
                    build_db
                } else {
                    cwd.join(path).to_str().unwrap().to_owned()
                }
            };
            cmd_line
        } else {
            let mysql_address = lgn_config::get::<String>("editor_srv.build_output")
                .unwrap()
                .and_then(|address| {
                    if address.starts_with("mysql:") {
                        Some(address)
                    } else {
                        None
                    }
                });

            mysql_address.unwrap_or_else(|| {
                lgn_config::get_absolute_path_or(
                    "editor_srv.build_output",
                    cwd.join("tests/sample-data/temp"),
                )
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
            })
        }
    };

    info!("Content Store: '{:?}'", content_store_path);
    info!("Build DB: '{}'", build_output_db_addr);

    let game_manifest_path = args.manifest.map_or_else(PathBuf::new, PathBuf::from);
    let assets_to_load = Vec::<ResourceTypeAndId>::new();

    let (trace_events_sender, trace_events_receiver) = broadcast::channel(1_000);

    let telemetry_guard = TelemetryGuardBuilder::default()
        .add_sink(LevelFilter::Info, BroadcastSink::new(trace_events_sender))
        .build()
        .expect("telemetry guard should be initialized once");

    let trace_events_receiver: TraceEventsReceiver = trace_events_receiver.into();

    let mut app = App::from_telemetry_guard(telemetry_guard);

    let mut streamer_plugin = StreamerPlugin::default();
    if args.enable_hw_encoding {
        streamer_plugin.enable_hw_encoding = true;
    }

    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
        1.0 / 60.0,
    )))
    .add_plugin(ScheduleRunnerPlugin::default())
    .insert_resource(DefaultTaskPoolOptions::new(1..=4))
    .add_plugin(CorePlugin::default())
    .add_plugin(AsyncPlugin::default())
    .insert_resource(AssetRegistrySettings::new(
        content_store_path.clone(),
        &game_manifest_path,
        assets_to_load,
    ))
    .add_plugin(AssetRegistryPlugin::default())
    .insert_resource(ResourceRegistrySettings::new(
        project_root,
        source_control_repository_index,
        repository_name,
        build_output_db_addr,
        content_store_path,
        args.compilers,
    ))
    .add_plugin(ResourceRegistryPlugin::default())
    .insert_resource(GRPCPluginSettings::new(listen_endpoint))
    .insert_resource(trace_events_receiver)
    .add_plugin(GRPCPlugin::default())
    .add_plugin(InputPlugin::default())
    .add_plugin(RendererPlugin::default())
    .add_plugin(streamer_plugin)
    .add_plugin(EditorPlugin::default())
    .insert_resource(ResourceBrowserSettings::new(scene))
    .add_plugin(ResourceBrowserPlugin::default())
    .add_plugin(ScenePlugin::new(None))
    .add_plugin(PropertyInspectorPlugin::default())
    .add_plugin(SourceControlPlugin::default())
    .add_plugin(TransformPlugin::default())
    .add_plugin(HierarchyPlugin::default())
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
