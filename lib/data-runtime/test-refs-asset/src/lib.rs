//! This module defines a test asset.
//!
//! It is used to test the data compilation process until we have a proper asset available.

use std::{any::Any, io, sync::Arc};

use byteorder::{LittleEndian, ReadBytesExt};
use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetRegistry, Reference, Resource, ResourceId, ResourceType,
    ResourceTypeAndId,
};
/// Asset temporarily used for testing.
///
/// To be removed once real asset types exist.
#[resource("refs_asset")]
pub struct RefsAsset {
    /// Test content.
    pub content: String,
    pub reference: Option<Reference<RefsAsset>>,
}

impl Asset for RefsAsset {
    type Loader = RefsAssetLoader;
}

/// [`RefsAsset`]'s asset creator temporarily used for testings.
///
/// To be removed once real asset types exists.
#[derive(Default)]
pub struct RefsAssetLoader {
    registry: Option<Arc<AssetRegistry>>,
}

impl AssetLoader for RefsAssetLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let nbytes = reader.read_u64::<LittleEndian>().expect("valid data");

        let mut content = vec![0u8; nbytes as usize];
        reader.read_exact(&mut content)?;
        let content = String::from_utf8(content).expect("valid utf8");
        let reference = read_maybe_reference::<RefsAsset>(reader)?;
        let asset = Box::new(RefsAsset { content, reference });
        Ok(asset)
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
        let asset = asset.downcast_mut::<RefsAsset>().unwrap();
        if let Some(reference) = &mut asset.reference {
            reference.activate(self.registry.as_ref().unwrap());
        }
    }
    fn register_registry(&mut self, registry: Arc<AssetRegistry>) {
        self.registry = Some(registry);
    }
}

fn read_maybe_reference<T>(
    reader: &mut dyn std::io::Read,
) -> Result<Option<Reference<T>>, std::io::Error>
where
    T: Any + Resource,
{
    let underlying_type = reader.read_u32::<LittleEndian>()?;
    if underlying_type == 0 {
        return Ok(None);
    }
    let underlying_id = reader.read_u128::<LittleEndian>()?;
    if underlying_id == 0 {
        return Ok(None);
    }
    Ok(Some(Reference::Passive(ResourceTypeAndId(
        ResourceType::from_raw(underlying_type),
        ResourceId::from_raw(underlying_id),
    ))))
}
