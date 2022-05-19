use lgn_data_runtime::{ResourcePathId, ResourceType};
use serde::{Deserialize, Serialize};

use crate::resource::ResourcePathName;

#[derive(Serialize, Deserialize)]
pub(crate) struct Metadata {
    pub(crate) name: ResourcePathName,
    pub(crate) type_name: String,
    pub(crate) type_id: ResourceType,
    pub(crate) dependencies: Vec<ResourcePathId>,
}

impl Metadata {
    pub(crate) fn rename(&mut self, name: &ResourcePathName) -> ResourcePathName {
        std::mem::replace(&mut self.name, name.clone())
    }
}
