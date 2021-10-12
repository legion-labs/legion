use crate::{resource::ResourcePathName, ResourcePathId};
use legion_data_runtime::ResourceChecksum;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::hash_map::DefaultHasher,
    convert::TryInto,
    fmt,
    hash::{Hash, Hasher},
};

/// Hash of resource's content.
///
/// Later it might include hashing of .meta file (excluding the resource name).
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceHash(u64);

impl ResourceHash {
    /// Retrieve value of resource hash as an unsigned 64 bit integer.
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for ResourceHash {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Serialize for ResourceHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let bytes = self.0.to_be_bytes();
            let hex = hex::encode(bytes);
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_u64(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for ResourceHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value = {
            if deserializer.is_human_readable() {
                let hex = String::deserialize(deserializer)?;
                let digits = hex::decode(hex).map_err(D::Error::custom)?;
                u64::from_be_bytes(digits.try_into().unwrap())
            } else {
                u64::deserialize(deserializer)?
            }
        };
        Ok(value.into())
    }
}

impl fmt::Debug for ResourceHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:016x}", self.0))
    }
}

impl fmt::Display for ResourceHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:016x}", self.0))
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Metadata {
    pub(crate) name: ResourcePathName,
    pub(crate) dependencies: Vec<ResourcePathId>,
    pub(crate) content_checksum: ResourceChecksum, // this needs to be updated on every asset change.
}

impl Metadata {
    pub(crate) fn rename(&mut self, name: &ResourcePathName) -> ResourcePathName {
        std::mem::replace(&mut self.name, name.clone())
    }

    pub(crate) fn new_with_dependencies(
        name: ResourcePathName,
        content_checksum: u128,
        deps: &[ResourcePathId],
    ) -> Self {
        Self {
            name,
            dependencies: deps.to_vec(),
            content_checksum: content_checksum.into(),
        }
    }

    pub(crate) fn resource_hash(&self) -> ResourceHash {
        let mut hasher = DefaultHasher::new();
        self.content_checksum.hash(&mut hasher);
        // todo(kstasik): include the hash of .meta content (excluding asset name) if it ever matters.
        hasher.finish().into()
    }
}
