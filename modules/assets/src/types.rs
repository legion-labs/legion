use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    fmt::LowerHex,
};

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
        let type_id = kind as u32;

        let internal = ((type_id as u64) << 32) | id as u64;
        Self {
            id: std::num::NonZeroU64::new(internal).unwrap(),
        }
    }

    /// Returns the type of the asset.
    pub fn to_type(&self) -> AssetType {
        let type_id = (u64::from(self.id) >> 32) as u32;
        type_id.try_into().unwrap()
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

/// Enumeration of runtime asset types.
///
/// `TODO`: for more flexibility it could be better to change this to asset registry.
#[derive(Clone, Copy, PartialEq, Debug, Hash)]
#[repr(u8)]
pub enum AssetType {
    /// Texture asset type.
    Texture,
    /// Material asset type.
    Material,
    /// Skeleton asset type.
    Skeleton,
}

impl TryFrom<u32> for AssetType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            value if value == Self::Texture as u32 => Ok(Self::Texture),
            value if value == Self::Material as u32 => Ok(Self::Material),
            value if value == Self::Skeleton as u32 => Ok(Self::Skeleton),
            _ => Err(()),
        }
    }
}
