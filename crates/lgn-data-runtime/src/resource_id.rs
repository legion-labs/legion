use lgn_content_store::indexing::{CompositeIndexer, IndexKey, StaticIndexer};
use lgn_data_model::ReflectionError;
use lgn_utils::DefaultHash;
use serde::ser::SerializeTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt::Write;
use std::path::PathBuf;
use std::{fmt, hash::Hash, str::FromStr};
use uuid::Uuid;

use crate::ResourceType;

/// Id of a runtime asset or source or derived resource.
///
/// We currently use fully random 128-bit UUIDs, to ensure uniqueness without
/// requiring a central authority. This allows creation of two `ResourceId` on
/// two separate machines and guarantee that we won't have any collision when
/// submitting those Resources on the source control.
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Hash)]
pub struct ResourceId(std::num::NonZeroU128);

impl ResourceId {
    /// Creates a new random id.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(std::num::NonZeroU128::new(Uuid::new_v4().as_u128()).unwrap())
    }
    /// Creates an explicit id, assuming that it is a runtime counter, not for
    /// serialization. The UUID 'version' is a non-standard value of 15.
    pub fn new_explicit(id: u64) -> Self {
        Self(
            std::num::NonZeroU128::new(
                uuid::Builder::from_u128(u128::from(id))
                    .set_version(unsafe { std::mem::transmute(0xF_u8) })
                    .as_uuid()
                    .as_u128(),
            )
            .unwrap(),
        )
    }

    /// Initialize from an existing, serialized, source.
    pub fn from_raw(id: u128) -> Self {
        Self(std::num::NonZeroU128::new(id).unwrap())
    }

    /// Initialize by hashing the contents of an object. We set 'Sha1' as UUID
    /// version even if our hash isn't really SHA-1.
    pub fn from_obj<T: Hash>(obj: &T) -> Self {
        let id = (*obj).default_hash_128();
        Self(
            std::num::NonZeroU128::new(
                uuid::Builder::from_u128(id)
                    .set_version(uuid::Version::Sha1)
                    .as_uuid()
                    .as_u128(),
            )
            .unwrap(),
        )
    }

    /// Returns a path of a resource.
    pub fn resource_path(&self) -> PathBuf {
        PathBuf::from(self)
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", Uuid::from_u128(self.0.get())))
    }
}

impl FromStr for ResourceId {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::from_str(s)?;
        Ok(Self::from_raw(id.as_u128()))
    }
}

impl From<ResourceId> for IndexKey {
    fn from(id: ResourceId) -> Self {
        id.0.get().into()
    }
}

impl From<IndexKey> for ResourceId {
    fn from(key: IndexKey) -> Self {
        Self::from_raw(key.into())
    }
}

impl TryFrom<u128> for ResourceId {
    type Error = ();

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Ok(Self::from_raw(value))
    }
}

impl From<&ResourceId> for PathBuf {
    fn from(id: &ResourceId) -> Self {
        let mut path = Self::new();
        let mut byte_text = String::with_capacity(2);
        for byte in id.0.get().to_be_bytes().into_iter().take(3) {
            write!(byte_text, "{:02x}", byte).unwrap();
            path.push(&byte_text);
            byte_text.clear();
        }
        path.push(id.to_string());
        path
    }
}

impl Serialize for ResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let id = Uuid::from_u128(self.0.get()).to_string();
            serializer.serialize_str(&id)
        } else {
            serializer.serialize_u128(self.0.get())
        }
    }
}

impl<'de> Deserialize<'de> for ResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = {
            if deserializer.is_human_readable() {
                let id = String::deserialize(deserializer)?;
                Uuid::from_str(&id).unwrap().as_u128()
            } else {
                u128::deserialize(deserializer)?
            }
        };
        Ok(Self::from_raw(id))
    }
}

/// FIXME: This should only be a temporary struct, we should be using the
/// `ResourceId` directly.
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct ResourceTypeAndId {
    /// The associated `ResourceType`.
    pub kind: ResourceType,

    /// The associated `ResourceId`.
    pub id: ResourceId,
}

lgn_data_model::implement_primitive_type_def!(
    ResourceTypeAndId,
    Result::<ResourceTypeAndId, ReflectionError>::Err(ReflectionError::InvalidFieldType(
        "Invalid default ResourceTypeAndId".into()
    ))
);

impl FromStr for ResourceTypeAndId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pair = s.trim_matches(|p| p == '(' || p == ')').split(',');
        let kind = pair
            .next()
            .ok_or("missing kind")?
            .parse::<ResourceType>()
            .map_err(|_err| "invalid resourcetype")?;
        let id = pair
            .next()
            .ok_or("missing id")?
            .parse::<ResourceId>()
            .map_err(|_err| "invalid resourceid")?;
        Ok(Self { kind, id })
    }
}

impl From<ResourceTypeAndId> for IndexKey {
    fn from(type_id: ResourceTypeAndId) -> Self {
        Self::compose(type_id.kind, type_id.id)
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<IndexKey> for ResourceTypeAndId {
    fn from(index_key: IndexKey) -> Self {
        let (kind, id) = index_key.decompose().unwrap();
        Self {
            kind: kind.into(),
            id: id.into(),
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<&IndexKey> for ResourceTypeAndId {
    fn from(index_key: &IndexKey) -> Self {
        let (kind, id) = index_key.decompose().unwrap();
        Self {
            kind: kind.into(),
            id: id.into(),
        }
    }
}

impl fmt::Display for ResourceTypeAndId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({},{})", self.kind, self.id))
    }
}

impl fmt::Debug for ResourceTypeAndId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({},{})", self.kind.as_pretty(), self.id))
    }
}

impl Serialize for ResourceTypeAndId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("{}", self))
        } else {
            let mut tup = serializer.serialize_tuple(2)?;
            tup.serialize_element(&self.kind)?;
            tup.serialize_element(&self.id)?;
            tup.end()
        }
    }
}

impl<'de> Deserialize<'de> for ResourceTypeAndId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let kind_id = String::deserialize(deserializer)?;
            Ok(Self::from_str(&kind_id).unwrap())
        } else {
            let (kind, id) = <(ResourceType, ResourceId)>::deserialize(deserializer)?;
            Ok(Self { kind, id })
        }
    }
}

/// Content store indexer that can be used to index by `ResourceTypeAndId`
pub type ResourceTypeAndIdIndexer = CompositeIndexer<StaticIndexer, StaticIndexer>;

/// Create a `new ResourceTypeAndIdIndexer`
pub fn new_resource_type_and_id_indexer() -> ResourceTypeAndIdIndexer {
    CompositeIndexer::new(
        StaticIndexer::new(std::mem::size_of::<ResourceType>()),
        StaticIndexer::new(std::mem::size_of::<ResourceId>()),
    )
}
