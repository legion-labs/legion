use core::fmt;
use std::{fmt::LowerHex, hash::Hash, str::FromStr};

use legion_content_store::ContentType;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Identifier of a resource.
///
/// Resources are identified in a path-like manner.
/// All `ResourcePathName` instances start with a separator **/**.
/// Each consecutive separator represents a directory while the component
/// after the last separator is the display name of the resource.
///
/// # Example
/// ```
/// # use legion_data_offline::resource::ResourcePathName;
/// let mut path = ResourcePathName::new("model");
/// path.push("npc");
/// path.push("dragon");
///
/// assert_eq!(path.to_string(), "/model/npc/dragon");
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ResourcePathName(String);

const SEPARATOR: char = '/';

impl ResourcePathName {
    /// New `ResourcePathName` in root directory.
    ///
    /// # Panics:
    ///
    /// Panics if name starts with a separator (is an absolute path).
    pub fn new(name: impl AsRef<str>) -> Self {
        assert_ne!(name.as_ref().chars().next().unwrap(), SEPARATOR);
        let mut s = String::from(SEPARATOR);
        s.push_str(name.as_ref());
        Self(s)
    }

    /// Extends self with path.
    ///
    /// # Panics:
    ///
    /// Panics if path starts with a separator (is an absolute path).
    pub fn push(&mut self, path: impl AsRef<str>) {
        assert_ne!(path.as_ref().chars().next().unwrap(), SEPARATOR);
        self.0.push(SEPARATOR);
        self.0.push_str(path.as_ref());
    }
}

impl ToString for ResourcePathName {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<String> for ResourcePathName {
    fn from(s: String) -> Self {
        assert_eq!(s.chars().next().unwrap(), SEPARATOR);
        Self(s)
    }
}

impl From<&str> for ResourcePathName {
    fn from(s: &str) -> Self {
        Self::from(s.to_owned())
    }
}

impl<T: AsRef<str>> From<&T> for ResourcePathName {
    fn from(s: &T) -> Self {
        Self::from(s.as_ref().to_owned())
    }
}

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
