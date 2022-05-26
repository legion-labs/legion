use serde::{Deserialize, Serialize};

use crate::resource::{metadata::Metadata, ResourcePathName};

#[derive(Serialize, Deserialize)]
pub(crate) struct OfflineContent {
    pub(crate) metadata: Metadata,
    pub(crate) content: Vec<u8>,
}

impl OfflineContent {
    pub(crate) fn new(name: ResourcePathName) -> Self {
        Self {
            metadata: Metadata {
                name,
                dependencies: Vec::new(),
            },
            content: Vec::new(),
        }
    }
}
