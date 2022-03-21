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
use lgn_app::prelude::*;
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_content_store::ContentStoreAddr;
use lgn_core::{CorePlugin, DefaultTaskPoolOptions};
use lgn_data_runtime::ResourceTypeAndId;
use lgn_graphics_data::GraphicsPlugin;
use lgn_graphics_renderer::RendererPlugin;
use lgn_input::InputPlugin;
use lgn_physics::{PhysicsPlugin, PhysicsSettingsBuilder};
use lgn_scripting::ScriptingPlugin;
use lgn_tracing::prelude::*;
use lgn_transform::prelude::*;
use sample_data::SampleDataPlugin;

#[cfg(feature = "standalone")]
mod standalone;

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

    let project_dir = {
        if let Some(params) = args.project {
            PathBuf::from(params)
        } else if let Some(key) = project_dir_setting {
            lgn_config::get_absolute_path_or(key, PathBuf::from(fallback_project_dir)).unwrap()
        } else {
            PathBuf::from(fallback_project_dir)
        }
    };

    let content_store_addr = {
        let cas = args
            .cas
            .map_or_else(|| project_dir.join("temp"), PathBuf::from);
        ContentStoreAddr::from(cas.to_str().unwrap())
    };

    let game_manifest = args.manifest.map_or_else(
        || project_dir.join("runtime").join("game.manifest"),
        PathBuf::from,
    );

    let mut assets_to_load = Vec::<ResourceTypeAndId>::new();

    // default root object is in sample data
    // /world/sample_1.ent

    let root_asset = args.root.as_deref().unwrap_or(fallback_root_asset);
    if let Ok(asset_id) = root_asset.parse::<ResourceTypeAndId>() {
        assets_to_load.push(asset_id);
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
        .add_plugin(TransformPlugin::default())
        .insert_resource(AssetRegistrySettings::new(
            content_store_addr,
            game_manifest,
            assets_to_load,
        ))
        .add_plugin(AssetRegistryPlugin::default())
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

        use lgn_async::AsyncPlugin;
        use lgn_grpc::{GRPCPlugin, GRPCPluginSettings};
        use lgn_streamer::StreamerPlugin;
        use lgn_window::WindowPlugin;

        let server_addr = {
            let url = args.addr.as_deref().unwrap_or_else(|| {
                lgn_config::get_or("runtime_srv.server_addr", "[::1]:50052").unwrap()
            });
            url.parse::<SocketAddr>()
                .unwrap_or_else(|err| panic!("Invalid server_addr '{}': {}", url, err))
        };

        app.add_plugin(WindowPlugin {
            add_primary_window: false,
            exit_on_close: false,
        })
        .add_plugin(AsyncPlugin::default())
        .insert_resource(GRPCPluginSettings::new(server_addr))
        .add_plugin(GRPCPlugin::default())
        .add_plugin(StreamerPlugin::default());
    }

    app
}

#[span_fn]
pub fn start_runtime(app: &mut App) {
    app.run();
}
