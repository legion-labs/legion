use std::any::Any;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{AssetRegistry, Handle, Resource, ResourceId, ResourceType};

/// A `ResourceReference` represents a reference to an external resource, that can be promoted to a handle
pub enum Reference<T>
where
    T: Any + Resource,
{
    /// Reference is not yet active, and is simply described as an id
    Passive((ResourceType, ResourceId)),

    /// Reference is active, and be accessed through a typed handle
    Active(Handle<T>),
}

impl<T> Reference<T>
where
    T: Any + Resource,
{
    /// Returns resource id associated with this Reference
    pub fn id(&self) -> (ResourceType, ResourceId) {
        match self {
            Self::Passive(resource_id) => *resource_id,
            Self::Active(handle) => handle.id(),
        }
    }

    /// Promote a reference to an active handle
    pub fn activate(&mut self, registry: &AssetRegistry) {
        if let Self::Passive(resource_id) = self {
            let handle = registry.get_untyped(*resource_id).unwrap();
            *self = Self::Active(handle.into());
        }
    }
}

impl<T> Serialize for Reference<T>
where
    T: Any + Resource,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Reference<T>
where
    T: Any + Resource,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // A Reference is always deserialized as passive, and will require activation
        let resource_id = <(ResourceType, ResourceId)>::deserialize(deserializer)?;
        Ok(Self::Passive(resource_id))
    }
}
