#![allow(unused_imports)]
#[path = "../offline/script.rs"]
mod script;
pub use script::*;

pub fn register_resource_types(
    resource_registry: &mut lgn_data_offline::resource::ResourceRegistryOptions,
) -> &mut lgn_data_offline::resource::ResourceRegistryOptions {
    resource_registry.add_type_mut::<Script>()
}
pub fn add_loaders(
    asset_registry: &mut lgn_data_runtime::AssetRegistryOptions,
) -> &mut lgn_data_runtime::AssetRegistryOptions {
    asset_registry.add_loader_mut::<Script>()
}
