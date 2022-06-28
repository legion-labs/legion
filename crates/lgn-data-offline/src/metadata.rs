use lgn_data_runtime::{ResourceDescriptor, ResourceId, ResourceTypeAndId};

use crate::ResourcePathName;

impl crate::offline::Metadata {
    /// Rename the metadata.
    pub fn rename(&mut self, name: &ResourcePathName) -> ResourcePathName {
        std::mem::replace(&mut self.name, name.clone())
    }

    /// Create an new instance with parameters.
    pub fn new(name: ResourcePathName, type_id: ResourceTypeAndId) -> Self {
        Self {
            name,
            type_id,
            dependencies: vec![],
        }
    }

    /// Create an new instance with parameters.
    pub fn new_default<T: ResourceDescriptor>() -> Self {
        Self {
            name: ResourcePathName::default(),
            type_id: ResourceTypeAndId {
                kind: T::TYPE,
                id: ResourceId::new(),
            },
            dependencies: vec![],
        }
    }
}
