#![allow(unused_imports)]
#[path = "../runtime/script.rs"]
mod script;
pub use script::*;

pub fn add_loaders(
    registry: &mut lgn_data_runtime::AssetRegistryOptions,
) -> &mut lgn_data_runtime::AssetRegistryOptions {
    registry.add_loader_mut::<Script>()
}
