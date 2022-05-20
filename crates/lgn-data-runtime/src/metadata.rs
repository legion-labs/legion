use lgn_data_model::{TypeDefinition, TypeReflection};
use serde::{Deserialize, Serialize};

use crate::{ResourcePathId, ResourcePathName, ResourceType};

/// The metadata represents all the basic properties that a resource has. Some resources
/// don't have metadata because they are embedded in other resources, or not visible to the user(s).
/// The metadata is serialized inside the resource.
#[derive(Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    /// The virtual path to the resource. This is only used in the editor for human consumption.
    /// It only provides a way to organize assets in the editor, and is not tied to any disk path.
    pub name: ResourcePathName,

    /// The typename of the resource this metadata points to.
    pub type_name: String,

    /// The type of the resource this metadata points to.
    pub type_id: ResourceType,

    /// Dependencies list, if any, of the resource.
    pub dependencies: Vec<ResourcePathId>,
}

impl Metadata {
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
}

impl Clone for Metadata {
    fn clone(&self) -> Self {
        Self {
            name: ResourcePathName::default(),
            type_name: self.type_name.clone(),
            type_id: self.type_id.clone(),
            dependencies: vec![],
        }
    }
}

impl TypeReflection for Metadata {
    fn get_type(&self) -> TypeDefinition {
        Self::get_type_def()
    }

    fn get_type_def() -> TypeDefinition {
        lgn_data_model::TypeDefinition::None
    }
}
