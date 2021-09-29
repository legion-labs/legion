use crate::{asset::AssetPathId, resource::ResourcePathName};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::hash_map::DefaultHasher,
    convert::TryInto,
    hash::{Hash, Hasher},
};

/// Hash of resource's content.
///
/// Later it might include hashing of .meta file (excluding the resource name).
pub type ResourceHash = u64;

#[derive(Serialize, Deserialize)]
pub(crate) struct Metadata {
    pub(crate) name: ResourcePathName,
    pub(crate) dependencies: Vec<AssetPathId>,
    pub(crate) content_checksum: ResourceChecksum, // this needs to be updated on every asset change.
}

impl Metadata {
    pub(crate) fn rename(&mut self, name: &ResourcePathName) -> ResourcePathName {
        std::mem::replace(&mut self.name, name.clone())
    }

    pub(crate) fn new_with_dependencies(
        name: ResourcePathName,
        content_checksum: i128,
        deps: &[AssetPathId],
    ) -> Self {
        Self {
            name,
            dependencies: deps.to_vec(),
            content_checksum: content_checksum.into(),
        }
    }

    pub(crate) fn resource_hash(&self) -> ResourceHash {
        let mut hasher = DefaultHasher::new();
        self.content_checksum.0.hash(&mut hasher);
        // todo(kstasik): include the hash of .meta content (excluding asset name) if it ever matters.
        hasher.finish()
    }
}

pub(crate) struct ResourceChecksum(i128);

impl From<i128> for ResourceChecksum {
    fn from(value: i128) -> Self {
        Self(value)
    }
}

impl Serialize for ResourceChecksum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let bytes = self.0.to_be_bytes();
            let hex = hex::encode(bytes);
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_i128(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for ResourceChecksum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value = {
            if deserializer.is_human_readable() {
                let hex = String::deserialize(deserializer)?;
                let digits = hex::decode(hex).map_err(D::Error::custom)?;
                i128::from_be_bytes(digits.try_into().unwrap())
            } else {
                i128::deserialize(deserializer)?
            }
        };
        Ok(value.into())
    }
}
