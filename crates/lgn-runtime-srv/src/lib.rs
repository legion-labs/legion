//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)

// crate-specific lint exceptions:
//#![allow()]

use std::{net::SocketAddr, path::PathBuf, str::FromStr};

use clap::Parser;
use generic_data::plugin::GenericDataPlugin;
use lgn_app::prelude::App;
#[cfg(not(feature = "standalone"))]
use lgn_app::{prelude::StartupStage, CoreStage};
#[cfg(not(feature = "standalone"))]
use lgn_asset_registry::AssetRegistryRequest;
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_async::{AsyncPlugin, TokioAsyncRuntime};
use lgn_config::RichPathBuf;
use lgn_core::{CorePlugin, DefaultTaskPoolOptions};
use lgn_data_runtime::ResourceTypeAndId;
#[cfg(not(feature = "standalone"))]
use lgn_ecs::prelude::{
    EventWriter, ExclusiveSystemDescriptorCoercion, IntoExclusiveSystem, Res, World,
};
use lgn_graphics_data::GraphicsPlugin;
use lgn_graphics_renderer::RendererPlugin;
use lgn_hierarchy::prelude::HierarchyPlugin;
use lgn_input::InputPlugin;
use lgn_physics::{PhysicsPlugin, PhysicsSettingsBuilder};
#[cfg(not(feature = "standalone"))]
use lgn_scene_plugin::SceneMessage;
use lgn_scene_plugin::ScenePlugin;
use lgn_scripting::ScriptingPlugin;
use lgn_streamer::StreamerPlugin;
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{info, span_fn};
use lgn_transform::prelude::TransformPlugin;
use sample_data::SampleDataPlugin;
use serde::Deserialize;

#[cfg(not(feature = "standalone"))]
mod grpc;
#[cfg(feature = "standalone")]
mod standalone;

#[cfg(not(feature = "standalone"))]
use crate::grpc::{GRPCServer, RuntimeServerCommand};

#[derive(Parser, Debug)]
#[clap(name = "Legion Labs runtime server")]
#[clap(
    about = "Server that will run with runtime data, and execute world simulation, ready to be streamed to a runtime client.",
    version,
    author
)]
struct Args {
    /// The address to listen on
    #[clap(long)]
    listen_endpoint: Option<SocketAddr>,
    /// Path to folder containing the project index
    #[clap(long)]
    project_root: Option<RichPathBuf>,
    /// Path to the game manifest
    #[clap(long)]
    manifest: Option<String>,
    /// Root object to load, usually a world
    #[clap(long)]
    root: Option<String>,
    /// Enable physics visual debugger
    #[clap(long)]
    physics_debugger: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    /// The endpoint to listen on.
    #[serde(default = "Config::default_listen_endpoint")]
    listen_endpoint: SocketAddr,

    /// The project root.
    project_root: Option<RichPathBuf>,

    /// The root asset.
    root: Option<String>,

    /// The streamer configuration.
    #[serde(default)]
    streamer: lgn_streamer::Config,

    /// Whether the program runs in AWS EC2 behind a NAT.
    #[serde(default)]
    enable_aws_ec2_nat_public_ipv4_auto_discovery: bool,
}

impl Config {
    fn default_listen_endpoint() -> SocketAddr {
        "[::1]:50052".parse().unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_endpoint: Self::default_listen_endpoint(),
            project_root: None,
            root: None,
            streamer: lgn_streamer::Config::default(),
            enable_aws_ec2_nat_public_ipv4_auto_discovery: false,
        }
    }
}

pub fn build_runtime() -> App {
    let telemetry_guard = TelemetryGuardBuilder::default()
        .build()
        .expect("telemetry guard should be initialized once");

    info!("Starting runtime server...");

    let mut app = App::from_telemetry_guard(telemetry_guard);

    let args = Args::parse();
    let cwd = std::env::current_dir().unwrap();
    let config: Config = lgn_config::get("runtime_server")
        .expect("failed to load config")
        .unwrap_or_default();

    let listen_endpoint = args.listen_endpoint.unwrap_or(config.listen_endpoint);

    info!("Listening on {}", listen_endpoint);

    let project_root = args
        .project_root
        .or(config.project_root)
        .expect("no `project_root` was specified");

    let project_root = if project_root.is_absolute() {
        project_root.to_path_buf()
    } else {
        cwd.join(project_root.as_ref())
    };

    // TODO: Figure out why this is needed.
    let project_root = if cfg!(windows) {
        PathBuf::from_str(&project_root.to_str().unwrap().replace("/", "\\")).unwrap()
    } else {
        project_root
    };

    info!("Project root: {}", project_root.display());

    let root_asset = if cfg!(feature = "standalone") {
        // default root object is in sample data
        // /world/sample_1.ent

        args.root.or(config.root).map(|asset| {
            asset
                .parse::<ResourceTypeAndId>()
                .expect("failed to parse root asset")
        })
    } else {
        None
    };

    info!(
        "Root: {}",
        root_asset.map_or("(none)".to_owned(), |asset| asset.to_string())
    );

    let tokio_rt = TokioAsyncRuntime::default();

    let game_manifest_path = if cfg!(feature = "standalone") {
        Some(args.manifest.map_or_else(PathBuf::new, PathBuf::from))
    } else {
        None
    };

    info!(
        "Manifest: {}",
        game_manifest_path
            .as_ref()
            .map_or("(none)".to_owned(), |path| path
                .as_path()
                .to_string_lossy()
                .to_string())
    );

    let mut assets_to_load = Vec::<ResourceTypeAndId>::new();
    if let Some(root_asset) = root_asset {
        assets_to_load.push(root_asset);
    }

    #[cfg(not(feature = "standalone"))]
    let streamer_plugin = StreamerPlugin {
        config: if config.enable_aws_ec2_nat_public_ipv4_auto_discovery {
            info!("Using AWS EC2 NAT public IPv4 auto-discovery.");

            let ipv4 = tokio_rt
                .block_on(lgn_online::cloud::get_aws_ec2_metadata_public_ipv4())
                .expect("failed to get AWS EC2 public IPv4");

            info!("AWS EC2 public IPv4: {}", ipv4);

            let mut streamer_config = config.streamer;

            info!("Adding NAT 1:1 public IPv4 to streamer configuration.");

            streamer_config.webrtc.nat_1to1_ips.push(ipv4.to_string());

            streamer_config
        } else {
            config.streamer
        },
    };

    info!("Streamer plugin config:\n{}", streamer_plugin.config);

    // physics settings
    let mut physics_settings = PhysicsSettingsBuilder::default();
    if args.physics_debugger {
        physics_settings = physics_settings.enable_visual_debugger(true);
    } else if let Some(enable_visual_debugger) =
        lgn_config::get("physics.enable_visual_debugger").unwrap()
    {
        physics_settings = physics_settings.enable_visual_debugger(enable_visual_debugger);
    }
    if let Some(length_tolerance) = lgn_config::get("physics.length_tolerance").unwrap() {
        physics_settings = physics_settings.length_tolerance(length_tolerance);
    }
    if let Some(speed_tolerance) = lgn_config::get("physics.speed_tolerance").unwrap() {
        physics_settings = physics_settings.speed_tolerance(speed_tolerance);
    }

    #[cfg(not(feature = "standalone"))]
    {
        use instant::Duration;
        use lgn_app::{ScheduleRunnerPlugin, ScheduleRunnerSettings};

        app
            // Start app with 60 fps
            .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            )))
            .add_plugin(ScheduleRunnerPlugin::default());
    }

    app.insert_resource(tokio_rt)
        .insert_resource(DefaultTaskPoolOptions::new(1..=4))
        .add_plugin(CorePlugin::default())
        .add_plugin(AsyncPlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(HierarchyPlugin::default())
        .insert_resource(AssetRegistrySettings::new(
            game_manifest_path,
            assets_to_load,
        ))
        .add_plugin(AssetRegistryPlugin::default())
        .add_plugin(ScenePlugin::new(root_asset))
        .add_plugin(GenericDataPlugin::default())
        .add_plugin(ScriptingPlugin::default())
        .add_plugin(SampleDataPlugin::default())
        .add_plugin(GraphicsPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(RendererPlugin::default())
        .insert_resource(physics_settings.build())
        .add_plugin(PhysicsPlugin::default());

    #[cfg(feature = "standalone")]
    standalone::build_standalone(&mut app);

    #[cfg(not(feature = "standalone"))]
    {
        use lgn_grpc::{GRPCPlugin, GRPCPluginSettings};
        use lgn_window::WindowPlugin;

        app.add_plugin(WindowPlugin {
            add_primary_window: false,
            exit_on_close: false,
        })
        .insert_resource(GRPCPluginSettings::new(listen_endpoint))
        .add_plugin(GRPCPlugin::default())
        .add_plugin(streamer_plugin);

        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            setup_runtime_grpc
                .exclusive_system()
                .before(lgn_grpc::GRPCPluginScheduling::StartRpcServer),
        )
        .add_system_to_stage(CoreStage::PreUpdate, rebroadcast_commands);
    }

    app
}

#[span_fn]
pub fn start_runtime(app: &mut App) {
    app.run();
}

#[cfg(not(feature = "standalone"))]
fn setup_runtime_grpc(world: &mut World) {
    let (command_sender, command_receiver) = crossbeam_channel::unbounded::<RuntimeServerCommand>();

    let grpc_server = GRPCServer::new(command_sender);
    let mut grpc_settings = world
        .get_resource_mut::<lgn_grpc::GRPCPluginSettings>()
        .expect("cannot retrieve resource GRPCPluginSettings from world");
    grpc_settings.register_service(grpc_server.service());

    world.insert_resource(command_receiver);
}

#[cfg(not(feature = "standalone"))]
fn rebroadcast_commands(
    command_receiver: Res<'_, crossbeam_channel::Receiver<RuntimeServerCommand>>,
    mut asset_registry_requests: EventWriter<'_, '_, AssetRegistryRequest>,
    mut scene_messsages: EventWriter<'_, '_, SceneMessage>,
) {
    while let Ok(command) = command_receiver.try_recv() {
        match command {
            RuntimeServerCommand::LoadManifest(manifest_id) => {
                asset_registry_requests.send(AssetRegistryRequest::LoadManifest(manifest_id));
            }
            RuntimeServerCommand::LoadRootAsset(root_id) => {
                asset_registry_requests.send(AssetRegistryRequest::LoadAsset(root_id));
                scene_messsages.send(SceneMessage::OpenScene(root_id));
            }
            RuntimeServerCommand::Pause => {
                //TODO
            }
        }
    }

    drop(command_receiver);
}
