use lgn_data_runtime::{ResourceDescriptor, ResourceType};

use crate::ResourcePathName;

impl crate::offline::Metadata {
    /// Rename the metadata.
    pub fn rename(&mut self, name: &ResourcePathName) -> ResourcePathName {
        std::mem::replace(&mut self.name, name.clone())
    }

    /// Create an new instance with parameters.
    pub fn new(name: ResourcePathName, type_name: &str, type_id: ResourceType) -> Self {
        Self {
            name,
            type_name: type_name.to_string(),
            type_id,
            dependencies: vec![],
        }
    }

    /// Create an new instance with parameters.
    pub fn new_default<T: ResourceDescriptor>() -> Self {
        Self {
            name: ResourcePathName::default(),
            type_name: T::TYPENAME.into(),
            type_id: T::TYPE,
            dependencies: vec![],
        }
    }
}
