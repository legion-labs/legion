use core::fmt;
use legion_content_store::ContentType;
use serde::{Deserialize, Serialize};
use std::{any::Any, fmt::LowerHex, hash::Hash, io};

/// A unique id of a runtime asset.
///
/// This 64 bit id encodes the following information:
/// - asset unique id - 32 bits
/// - [`AssetType`] - 32 bits
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct AssetId {
    id: std::num::NonZeroU64,
}

impl AssetId {
    /// Creates an asset id of a given type.
    pub fn new(kind: AssetType, id: u32) -> Self {
        let internal = kind.stamp(id as u64);
        Self {
            id: std::num::NonZeroU64::new(internal).unwrap(),
        }
    }

    /// Creates an asset id from a raw hash value.
    pub fn from_hash_id(id: u64) -> Option<Self> {
        std::num::NonZeroU64::new(id).map(|id| Self { id })
    }

    /// Returns the type of the asset.
    pub fn asset_type(&self) -> AssetType {
        let type_id = (u64::from(self.id) >> 32) as u32;
        AssetType::from_raw(type_id)
    }
}

impl LowerHex for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::LowerHex::fmt(&self.id, f)
    }
}

impl ToString for AssetId {
    fn to_string(&self) -> String {
        self.id.to_string()
    }
}

/// Type id of a runtime asset.
pub type AssetType = ContentType;

/// Types implementing `Asset` represent non-mutable runtime data.
pub trait Asset: Any + Send {
    /// Cast to &dyn Any type.
    fn as_any(&self) -> &dyn Any;

    /// Cast to &mut dyn Any type.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Asset initialization executed after the asset and all its dependencies
    /// have been loaded.
    fn load_init(&mut self) {
        // by default, do nothing
    }
}

/// An interface allowing to create and initialize assets.
pub trait AssetLoader {
    /// Asset loading interface.
    fn load(
        &mut self,
        kind: AssetType,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, io::Error>;
}
