//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)

// crate-specific lint exceptions:
//#![allow()]

use std::path::PathBuf;

use clap::Parser;
use generic_data::plugin::GenericDataPlugin;
#[cfg(not(feature = "standalone"))]
use lgn_app::prelude::StartupStage;
use lgn_app::{prelude::App, CoreStage};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_async::AsyncPlugin;
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
use lgn_scene_plugin::ScenePlugin;
use lgn_scripting::ScriptingPlugin;
use lgn_tracing::prelude::span_fn;
use lgn_transform::prelude::TransformPlugin;
use sample_data::SampleDataPlugin;

#[cfg(not(feature = "standalone"))]
mod grpc;
#[cfg(feature = "standalone")]
mod standalone;

#[cfg(not(feature = "standalone"))]
use crate::grpc::{GRPCServer, RuntimeServerCommand};

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

pub fn build_runtime(
    project_dir_setting: Option<&str>,
    fallback_project_dir: &str,
    fallback_root_asset: &str,
) -> App {
    let args = Args::parse();

    let game_manifest = if cfg!(feature = "standalone") {
        let project_dir = {
            if let Some(params) = args.project {
                PathBuf::from(params)
            } else if let Some(key) = project_dir_setting {
                lgn_config::get_absolute_path_or(key, PathBuf::from(fallback_project_dir)).unwrap()
            } else {
                PathBuf::from(fallback_project_dir)
            }
        };

        Some(args.manifest.map_or_else(
            || project_dir.join("runtime").join("game.manifest"),
            PathBuf::from,
        ))
    } else {
        None
    };

    let mut assets_to_load = Vec::<ResourceTypeAndId>::new();

    let root_asset = if cfg!(feature = "standalone") {
        // default root object is in sample data
        // /world/sample_1.ent

        args.root
            .as_deref()
            .unwrap_or(fallback_root_asset)
            .parse::<ResourceTypeAndId>()
            .ok()
    } else {
        None
    };

    if let Some(root_asset) = root_asset {
        assets_to_load.push(root_asset);
    }

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

    let mut app = App::default();

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

    app.insert_resource(DefaultTaskPoolOptions::new(1..=4))
        .add_plugin(CorePlugin::default())
        .add_plugin(AsyncPlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(HierarchyPlugin::default())
        .insert_resource(AssetRegistrySettings::new(game_manifest, assets_to_load))
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
        use std::net::SocketAddr;

        use lgn_grpc::{GRPCPlugin, GRPCPluginSettings};
        use lgn_streamer::StreamerPlugin;
        use lgn_window::WindowPlugin;

        let server_addr = {
            let url = args.addr.unwrap_or_else(|| {
                lgn_config::get_or("runtime_srv.server_addr", "[::1]:50052".to_owned()).unwrap()
            });
            url.parse::<SocketAddr>()
                .unwrap_or_else(|err| panic!("Invalid server_addr '{}': {}", url, err))
        };

        app.add_plugin(WindowPlugin {
            add_primary_window: false,
            exit_on_close: false,
        })
        .insert_resource(GRPCPluginSettings::new(server_addr))
        .add_plugin(GRPCPlugin::default())
        .add_plugin(StreamerPlugin::default());

        app.add_event::<RuntimeServerCommand>()
            .add_startup_system_to_stage(
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
    mut command_event_writer: EventWriter<'_, '_, RuntimeServerCommand>,
) {
    while let Ok(command) = command_receiver.try_recv() {
        command_event_writer.send(command);
    }

    drop(command_receiver);
}
