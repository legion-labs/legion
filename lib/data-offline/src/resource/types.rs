use core::fmt;
use std::{
    fmt::LowerHex,
    hash::Hash,
    path::{Path, PathBuf},
    str::FromStr,
};

use legion_content_store::ContentType;
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
        let rand_id: u64 = rand::thread_rng().gen();

        let internal = kind.stamp(rand_id);
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
pub type ResourceType = ContentType;
