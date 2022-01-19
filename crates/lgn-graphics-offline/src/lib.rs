//! Offline Graphics

// crate-specific lint exceptions:
#![warn(missing_docs)]

use lgn_data_offline::resource::ResourceRegistryOptions;

pub mod material;
pub use material::Material;

pub mod psd;
pub use crate::psd::PsdFile;

pub mod texture;
pub use texture::Texture;

/// Register crate's resource types to resource registry
pub fn register_resource_types(registry: &mut ResourceRegistryOptions) {
    registry
        .add_type_mut::<Material>()
        .add_type_mut::<PsdFile>()
        .add_type_mut::<Texture>();
}
