use core::fmt;
use std::{
    collections::hash_map::DefaultHasher,
    fmt::LowerHex,
    hash::{Hash, Hasher},
    num::ParseIntError,
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

/// Identifier of a path in a build graph.
///
/// Considering a build graph where nodes represent *resources* and edges representing *transformations* between resources
/// the `ResourcePathId` uniqely identifies any resource/node in the build graph.
///
/// A tuple (`ResourceType`, `ResourceType`) identifies a transformation type between two resource types.
///
/// `ResourcePathId` identifies a concrete resource with a `ResourceId` - that also defines `ResourceType` of the resource.
/// It also defines an ordered list of `ResourceType`s this source resource must be transformed into during the data build process.
///
/// # Example
///
/// The following example illustrates creation of *source resource* containing geometry and
/// definition of a path representing a *derived resource* of a runtime geometry data after LOD-generation process.
///
/// ```no_run
/// # use legion_resources::{ResourceRegistry, ResourceType};
/// # use legion_resources::ResourcePathId;
/// # use legion_resources::ResourceName;
/// # use legion_resources::test_resource;
/// # use legion_resources::Project;
/// # use std::path::PathBuf;
/// # let mut resources = ResourceRegistry::default();
/// # let mut project = Project::create_new(&PathBuf::new()).unwrap();
/// # pub const SOURCE_GEOMETRY: ResourceType = ResourceType::new(b"src_geom");
/// # pub const LOD_GEOMETRY: ResourceType = ResourceType::new(b"lod_geom");
/// # pub const BINARY_GEOMETRY: ResourceType = ResourceType::new(b"bin_geom");
/// // create a resource and add it to the project
/// let resource_handle = resources.new_resource(SOURCE_GEOMETRY).unwrap();
/// let resource_id = project.add_resource(ResourceName::from("new resource"),
///                              SOURCE_GEOMETRY, &resource_handle, &mut resources).unwrap();
///
/// // create a resource path
/// let source_path = ResourcePathId::from(resource_id);
/// let target = source_path.transform(LOD_GEOMETRY).transform(BINARY_GEOMETRY);
/// ```
#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, PartialOrd, Ord)]
pub struct ResourcePathId {
    source: ResourceId,
    transforms: Vec<ResourceType>,
}

impl From<ResourceId> for ResourcePathId {
    fn from(id: ResourceId) -> Self {
        Self {
            source: id,
            transforms: vec![],
        }
    }
}

impl fmt::Display for ResourcePathId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.source))?;
        for kind in &self.transforms {
            f.write_fmt(format_args!("|{}", kind))?;
        }
        Ok(())
    }
}

impl FromStr for ResourcePathId {
    type Err = ParseIntError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let end = s.find('|').unwrap_or(s.len());
        let source = ResourceId::from_str(&s[0..end])?;
        s = &s[end..];

        let mut transforms = vec![];
        while !s.is_empty() {
            s = &s[1..]; // skip '|'
            let end = s.find('|').unwrap_or(s.len());
            let t = u32::from_str(&s[0..end])?;
            transforms.push(ResourceType::from_raw(t));
            s = &s[end..];
        }
        Ok(Self { source, transforms })
    }
}

impl ResourcePathId {
    /// Changes the `ResourceType` of the path by appending a new transformation to the build path
    /// represented by this `ResourcePathId`.
    pub fn transform(&self, kind: ResourceType) -> Self {
        let mut cloned = self.clone();
        cloned.transforms.push(kind);
        cloned
    }

    /// Returns `ResourceType` of the resource identified by this path.
    pub fn resource_type(&self) -> ResourceType {
        if self.transforms.is_empty() {
            self.source.resource_type()
        } else {
            self.transforms[self.transforms.len() - 1]
        }
    }

    /// Deprecated: temporarily used by the compilers to load resources.
    pub fn source_resource_deprecated(&self) -> ResourceId {
        self.source
    }

    /// Returns true if the path identifies a `source resource`.
    ///
    /// Source resource has no transformations attached to it and is backed by user-defined data.
    pub fn is_source(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Returns the last transformation that must be applied to produce the resource.
    ///
    /// Returns None if self is a `source resource`.
    pub fn last_transform(&self) -> Option<(ResourceType, ResourceType)> {
        match self.transforms.len() {
            0 => None,
            1 => Some((self.source.resource_type(), self.transforms[0])),
            _ => {
                let len = self.transforms.len();
                Some((self.transforms[len - 2], self.transforms[len - 1]))
            }
        }
    }

    /// Returns a `ResourcePathId` that represents a direct dependency in the build graph.
    ///
    /// None if self represents a source dependency.
    pub fn direct_dependency(&self) -> Option<Self> {
        if self.is_source() {
            return None;
        }
        let mut dependency = self.clone();
        dependency.transforms.pop();
        Some(dependency)
    }

    /// Returns a hash of the `ResourceIdPath`.
    pub fn hash_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
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

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{test_resource, ResourceId, ResourcePathId};

    #[test]
    fn resource_path_name() {
        let source = ResourceId::generate_new(test_resource::TYPE_ID);

        let path_a = ResourcePathId::from(source);
        let path_b = path_a.transform(test_resource::TYPE_ID);

        let name_a = format!("{}", path_a);
        assert_eq!(path_a, ResourcePathId::from_str(&name_a).unwrap());

        let name_b = format!("{}", path_b);
        assert_eq!(path_b, ResourcePathId::from_str(&name_b).unwrap());
    }
}
