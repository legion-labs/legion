//! A module providing runtime material related functionality.

use std::{any::Any, convert::TryFrom};

use legion_data_runtime::{resource, Asset, AssetLoader, Resource, ResourceId, ResourceType};

use byteorder::{LittleEndian, ReadBytesExt};

/// Runtime material.
#[resource("runtime_material")]
pub struct Material {
    /// Albedo texture reference.
    pub albedo: Option<ResourceId>,
    /// Normal texture reference.
    pub normal: Option<ResourceId>,
    /// Roughness texture reference.
    pub roughness: Option<ResourceId>,
    /// Metalness texture reference.
    pub metalness: Option<ResourceId>,
}

impl Asset for Material {
    type Loader = MaterialLoader;
}

/// Creator of [`Material`].
#[derive(Default)]
pub struct MaterialLoader {}

fn read_asset_id(reader: &mut dyn std::io::Read) -> Result<Option<ResourceId>, std::io::Error> {
    let underlying = reader.read_u128::<LittleEndian>()?;
    Ok(ResourceId::try_from(underlying).ok().map(ResourceId::from))
}

impl AssetLoader for MaterialLoader {
    fn load(
        &mut self,
        _kind: ResourceType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, std::io::Error> {
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

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
