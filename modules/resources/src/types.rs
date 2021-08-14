use core::fmt;
use std::{
    fmt::LowerHex,
    hash::Hash,
    path::{Path, PathBuf},
    str::FromStr,
};

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Name identifier of a resource.
pub type ResourceName = PathBuf;

/// Temporarily a reference to `ResourceName` to silence lints.
pub type ResourceNameRef = Path;

/// Extension of a resource file.
pub const RESOURCE_EXT: &str = "blob";

/// A unique id of an offline resource.
///
/// This 64 bit id encodes the following information:
/// - resource unique id - 32 bits
/// - [`ResourceType`] - 32 bits
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Serialize, Deserialize, Hash)]
pub struct ResourceId {
    id: std::num::NonZeroU64,
}

impl ResourceId {
    /// Creates a new random id.
    pub fn generate_new(kind: ResourceType) -> Self {
        let rand_id: u32 = rand::thread_rng().gen();
        let type_id = kind.0 as u32;

        let internal = ((type_id as u64) << 32) | rand_id as u64;
        Self {
            id: std::num::NonZeroU64::new(internal).unwrap(),
        }
    }

    /// Returns the type of the resource.
    pub fn resource_type(&self) -> ResourceType {
        let type_id = (u64::from(self.id) >> 32) as u32;
        ResourceType::from_raw(type_id)
    }

    /// TODO: This should be removed. Exposed for serialization for now.
    pub fn get_internal(&self) -> u64 {
        self.id.get()
    }

    /// TODO: This should be removed. Exposed for deserialization for now.
    pub fn from_raw(internal: u64) -> Option<Self> {
        let id = std::num::NonZeroU64::new(internal)?;
        Some(Self { id })
    }
}

impl LowerHex for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::LowerHex::fmt(&self.id, f)
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#016x}", self.id))
    }
}

impl FromStr for ResourceId {
    type Err = std::num::ParseIntError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s.trim_start_matches("0x");
        let id = u64::from_str_radix(s, 16)?;
        if id == 0 {
            Err("Z".parse::<i32>().expect_err("ParseIntError"))
        } else {
            // SAFETY: id is not zero in this else clause.
            let id = unsafe { std::num::NonZeroU64::new_unchecked(id) };
            Ok(Self { id })
        }
    }
}

/// Type identifier of an offline resource.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Debug)]
pub struct ResourceType(u32);

impl ResourceType {
    const CRC32_ALGO: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

    const fn crc32(v: &[u8]) -> u32 {
        Self::CRC32_ALGO.checksum(v)
    }

    /// Creates a new 32 bit resource type id from series of bytes.
    ///
    /// It is recommended to use this method to define a public constant
    /// which can be used to identify an resource type.
    pub const fn new(v: &[u8]) -> Self {
        // TODO: A std::num::NonZeroU32 would be more suitable as an internal representation
        // however a value of 0 is as likely as any other value returned by `crc32`
        // and const-fn-friendly panic is not available yet.
        // See https://github.com/rust-lang/rfcs/pull/2345.
        Self(Self::crc32(v))
    }

    /// Creates a 32 bit resource type id from a non-zero integer.
    pub fn from_raw(v: u32) -> Self {
        Self(v)
    }
}
