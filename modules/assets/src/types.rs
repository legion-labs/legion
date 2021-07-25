use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fmt::LowerHex,
    hash::{Hash, Hasher},
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
        let type_id = kind.0;

        let internal = ((type_id as u64) << 32) | id as u64;
        Self {
            id: std::num::NonZeroU64::new(internal).unwrap(),
        }
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

/// Returns the hash of the provided data.
pub fn compute_asset_checksum(data: &[u8]) -> i128 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish() as i128
}

/// Type id of a runtime asset.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetType(u32);

impl AssetType {
    const CRC32_ALGO: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

    const fn crc32(v: &[u8]) -> u32 {
        Self::CRC32_ALGO.checksum(v)
    }

    /// Creates a new 32 bit asset type id from series of bytes.
    ///
    /// It is recommended to use this method to define a public constant
    /// which can be used to identify an asset type.
    pub const fn new(v: &[u8]) -> Self {
        // TODO: A std::num::NonZeroU32 would be more suitable as an internal representation
        // however a value of 0 is as likely as any other value returned by `crc32`
        // and const-fn-friendly panic is not available yet.
        // See https://github.com/rust-lang/rfcs/pull/2345.
        Self(Self::crc32(v))
    }

    /// Creates a 32 bit asset type id from a non-zero integer.
    pub fn from_raw(v: u32) -> Self {
        Self(v)
    }
}
