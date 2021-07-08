use core::fmt;
use std::{
    convert::{TryFrom, TryInto},
    fmt::LowerHex,
    hash::Hash,
    path::PathBuf,
    u8,
};

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Name identifier of a resource.
pub type ResourcePath = PathBuf;

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
        let type_id = kind as u32;

        let internal = ((type_id as u64) << 32) | rand_id as u64;
        Self {
            id: std::num::NonZeroU64::new(internal).unwrap(),
        }
    }

    /// Returns the type of the resource.
    pub fn to_type(&self) -> ResourceType {
        let type_id = (u64::from(self.id) >> 32) as u32;
        type_id.try_into().unwrap()
    }
}

impl LowerHex for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::LowerHex::fmt(&self.id, f)
    }
}

/// A type identifier of an offline resource.
///
/// `TODO`: this needs pulling out into a lower-level crate.
/// `TODO`: for more flexibility it could be better to change this to resource registry.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug, Hash)]
#[repr(u8)]
pub enum ResourceType {
    /// Texture resource type.
    Texture,
    /// Metarial resource type.
    Material,
    /// Geometry resource type.
    Geometry,
    /// Skeleton resource type.
    Skeleton,
    /// Actor resource type.
    Actor,
}

impl TryFrom<u32> for ResourceType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            value if value == Self::Texture as u32 => Ok(Self::Texture),
            value if value == Self::Material as u32 => Ok(Self::Material),
            value if value == Self::Geometry as u32 => Ok(Self::Geometry),
            value if value == Self::Skeleton as u32 => Ok(Self::Skeleton),
            value if value == Self::Actor as u32 => Ok(Self::Actor),
            _ => Err(()),
        }
    }
}
