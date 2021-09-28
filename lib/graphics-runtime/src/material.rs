//! A module providing runtime material related functionality.

use std::convert::TryFrom;

use legion_data_runtime::{Asset, AssetId, AssetLoader, AssetType, ContentId};

use byteorder::{LittleEndian, ReadBytesExt};

/// Type id.
pub const TYPE_ID: AssetType = AssetType::new(b"runtime_material");

/// Runtime material.
#[derive(Asset)]
pub struct Material {
    /// Albedo texture reference.
    pub albedo: Option<AssetId>,
    /// Normal texture reference.
    pub normal: Option<AssetId>,
    /// Roughness texture reference.
    pub roughness: Option<AssetId>,
    /// Metalness texture reference.
    pub metalness: Option<AssetId>,
}

/// Creator of [`Material`].
#[derive(Default)]
pub struct MaterialCreator {}

fn read_asset_id(reader: &mut dyn std::io::Read) -> Result<Option<AssetId>, std::io::Error> {
    let underlying = reader.read_u128::<LittleEndian>()?;
    let id = ContentId::try_from(underlying)
        .map_err(|_e| std::io::Error::new(std::io::ErrorKind::InvalidData, ""))?;
    let id = AssetId::try_from(id).ok();
    Ok(id)
}

impl AssetLoader for MaterialCreator {
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, std::io::Error> {
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

    fn load_init(&mut self, _asset: &mut (dyn Asset + Send + Sync)) {}
}
