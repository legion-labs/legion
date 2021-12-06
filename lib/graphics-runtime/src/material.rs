//! A module providing runtime material related functionality.

use std::{any::Any, io, sync::Arc};

use byteorder::{LittleEndian, ReadBytesExt};
use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetRegistry, Reference, Resource, ResourceId,
};

use crate::Texture;

/// Runtime material.
#[resource("runtime_material")]
pub struct Material {
    /// Albedo texture reference.
    pub albedo: Reference<Texture>,
    /// Normal texture reference.
    pub normal: Reference<Texture>,
    /// Roughness texture reference.
    pub roughness: Reference<Texture>,
    /// Metalness texture reference.
    pub metalness: Reference<Texture>,
}

impl Asset for Material {
    type Loader = MaterialLoader;
}

/// Creator of [`Material`].
#[derive(Default)]
pub struct MaterialLoader {
    registry: Option<Arc<AssetRegistry>>,
}

fn read_asset_id<T>(reader: &mut dyn std::io::Read) -> Result<Reference<T>, std::io::Error>
where
    T: Any + Resource,
{
    let underlying = reader.read_u128::<LittleEndian>()?;
    match ResourceId::try_from(underlying) {
        Ok(resource_id) => Ok(Reference::Passive(resource_id)),
        Err(_err) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "failed to read asset id",
        )),
    }
}

impl AssetLoader for MaterialLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let albedo = read_asset_id(reader)?;
        let normal = read_asset_id(reader)?;
        let roughness = read_asset_id(reader)?;
        let metalness = read_asset_id(reader)?;

        let output = Material {
            albedo,
            normal,
            roughness,
            metalness,
        };

        Ok(Box::new(output))
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
        let material = asset.downcast_mut::<Material>().unwrap();
        println!("runtime material loaded");

        // activate references
        if let Some(registry) = &self.registry {
            material.albedo.activate(registry);
            material.normal.activate(registry);
            material.roughness.activate(registry);
            material.metalness.activate(registry);
        }
    }

    fn register_registry(&mut self, registry: Arc<AssetRegistry>) {
        self.registry = Some(registry);
    }
}
