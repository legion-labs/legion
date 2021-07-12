use crate::{ResourceId, ResourcePath};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Hash of resource's content.
///
/// Later it might include hashing of .meta file (excluding the resource name).
pub type ResourceHash = u64;

#[derive(Serialize, Deserialize)]
pub(crate) struct Metadata {
    pub(crate) name: ResourcePath,
    pub(crate) build_deps: Vec<ResourceId>,
    pub(crate) content_checksum: i128, // this needs to be updated on every asset change.
}

impl Metadata {
    pub(crate) fn rename(&mut self, name: ResourcePath) -> ResourcePath {
        std::mem::replace(&mut self.name, name)
    }
}

impl Metadata {
    pub(crate) fn new_with_dependencies(
        name: ResourcePath,
        content_checksum: i128,
        deps: &[ResourceId],
    ) -> Self {
        Self {
            name,
            build_deps: deps.to_vec(),
            content_checksum,
        }
    }

    pub(crate) fn resource_hash(&self) -> ResourceHash {
        let mut hasher = DefaultHasher::new();
        self.content_checksum.hash(&mut hasher);
        // todo(kstasik): include the hash of .meta content (excluding asset name) if it ever matters.
        hasher.finish()
    }
}
