use crate::ResourcePathName;
use lgn_data_runtime::{ResourcePathId, ResourceTypeAndId};

/// The metadata represents all the basic properties that a resource has. Some resources
/// don't have metadata because they are embedded in other resources, or not visible to the user(s).
/// The metadata is serialized inside the resource.
#[legion(no_default, offline_only, hidden)]
pub struct Metadata {
    /// The virtual path to the resource. This is only used in the editor for human consumption.
    /// It only provides a way to organize assets in the editor, and is not tied to any disk path.
    pub name: ResourcePathName,

    /// The type of the resource this metadata points to.
    pub type_id: ResourceTypeAndId,

    /// Dependencies list, if any, of the resource. TODO: remove.
    pub dependencies: Vec<ResourcePathId>,
}
