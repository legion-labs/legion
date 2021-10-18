include!(concat!(env!("OUT_DIR"), "/data.rs"));

use legion_data_runtime::AssetRegistryOptions;

/// Register crate's asset types to asset registry
pub fn add_loaders(registry: AssetRegistryOptions) -> AssetRegistryOptions {
    registry.add_loader::<TestEntity>()
}
