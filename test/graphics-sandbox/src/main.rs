use std::time::Duration;

use legion_app::{App, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use legion_async::AsyncPlugin;
use legion_core::CorePlugin;
use legion_data_runtime::ResourceId;
use legion_renderer::RendererPlugin;
use legion_resource_registry::{ResourceRegistryPlugin, ResourceRegistrySettings};

fn main() {
    let project_folder = "test/sample_data";
    let content_store_addr = "test/sample_data/temp";
    let game_manifest = "test/sample_data/runtime/game.manifest";
    let assets_to_load: Vec<ResourceId> = Vec::new();
    
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(CorePlugin::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(AsyncPlugin {})
        .insert_resource(ResourceRegistrySettings::new(project_folder))
        .add_plugin(ResourceRegistryPlugin::default())
        .insert_resource(AssetRegistrySettings::new(
            content_store_addr,
            game_manifest,
            assets_to_load,
        ))
        .add_plugin(AssetRegistryPlugin::default())
        .add_plugin(RendererPlugin::default())
        .run();
}
