use crate::{AssetRegistry, Handle, Resource, ResourceId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;

/// A `ResourceReference` represents a reference to an external resource, that can be promoted to a handle
pub enum Reference<T>
where
    T: Any + Resource,
{
    /// Reference is not yet active, and is simply described as an id
    Passive(ResourceId),

    /// Reference is unset
    None,

    /// Reference is active, and be accessed through a typed handle
    Active(Handle<T>),
}

impl<T> Reference<T>
where
    T: Any + Resource,
{
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
        let resource_id = match self {
            Self::Passive(resource_id) => Some(*resource_id),
            Self::None => None,
            Self::Active(handle) => Some(handle.id()),
        };
        resource_id.serialize(serializer)
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
        // Unless unset (None), a Reference will be deserialized as passive, and will require activation
        let resource_id = Option::<ResourceId>::deserialize(deserializer)?;
        match resource_id {
            Some(resource_id) => Ok(Self::Passive(resource_id)),
            None => Ok(Self::None),
        }
    }
}
