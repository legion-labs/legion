//! Runtime Graphics

// crate-specific lint exceptions:
#![warn(missing_docs)]

pub mod material;
pub use material::Material;

pub mod texture;
use lgn_data_runtime::AssetRegistryOptions;
pub use texture::Texture;

/// Register crate's asset types to asset registry
pub fn add_loaders(registry: &mut AssetRegistryOptions) {
    registry
        .add_loader_mut::<Material>()
        .add_loader_mut::<Texture>();
}
