use lgn_data_runtime::ResourcePathId;
use serde::{Deserialize, Serialize};

use crate::resource::ResourcePathName;

#[derive(Serialize, Deserialize)]
pub(crate) struct Metadata {
    pub(crate) name: ResourcePathName,
    pub(crate) dependencies: Vec<ResourcePathId>,
}

impl Metadata {
    pub(crate) fn serialize(&self, writer: impl std::io::Write) {
        bincode::serialize_into(writer, &self).expect("failed to serialize metadata");
    }

    pub(crate) fn deserialize(reader: impl std::io::Read) -> Self {
        bincode::deserialize_from(reader).expect("failed to decode metadata contents")
    }

    pub(crate) fn rename(&mut self, name: &ResourcePathName) -> ResourcePathName {
        std::mem::replace(&mut self.name, name.clone())
    }
}

/// Write serialized form of metadata
pub fn serialize_metadata(
    name: ResourcePathName,
    dependencies: Vec<ResourcePathId>,
    writer: impl std::io::Write,
) {
    let metadata = Metadata { name, dependencies };
    metadata.serialize(writer);
}

/// Read over serialized form of metadata, advancing the reader
pub fn deserialize_and_skip_metadata(reader: impl std::io::Read) {
    let _metadata = Metadata::deserialize(reader);
}
