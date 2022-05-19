use lgn_data_runtime::ResourceType;
use serde::{Deserialize, Serialize};

use crate::resource::{metadata::Metadata, ResourcePathName};

#[derive(Serialize, Deserialize)]
pub(crate) struct OfflineContent {
    pub(crate) metadata: Metadata,
    pub(crate) content: Vec<u8>,
}

impl OfflineContent {
    pub(crate) fn new(name: ResourcePathName, type_name: &str, type_id: ResourceType) -> Self {
        Self {
            metadata: Metadata {
                name,
                type_name: type_name.to_owned(),
                type_id,
                dependencies: Vec::new(),
            },
            content: Vec::new(),
        }
    }
}
