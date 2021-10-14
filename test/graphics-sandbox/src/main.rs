use legion_app::App;
use legion_asset_registry::AssetRegistryPlugin;
use legion_async::AsyncPlugin;
use legion_renderer::RendererPlugin;
use legion_resource_registry::{ResourceRegistryPlugin, ResourceRegistrySettings};

fn main() {
    let project_folder = "test/sample_data";

    App::new()
        .add_plugin(AsyncPlugin {})
        .insert_resource(ResourceRegistrySettings::new(project_folder))
        .add_plugin(ResourceRegistryPlugin::default())
        .add_plugin(AssetRegistryPlugin::default())
        .add_plugin(RendererPlugin::default())
        .run();
}
