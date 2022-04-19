use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Handle, HandleUntyped, Resource, ResourceTypeAndId};

/// A `ReferenceUntyped` represents a reference to an external resource, that
/// can be promoted to a handle
pub enum ReferenceUntyped {
    /// Reference is not yet active, and is simply described as an id
    Passive(ResourceTypeAndId),
    /// Reference is active, and be accessed through a typed handle
    Active((ResourceTypeAndId, HandleUntyped)),
}

impl Clone for ReferenceUntyped {
    fn clone(&self) -> Self {
        match self {
            Self::Passive(resource_id) => Self::Passive(*resource_id),
            Self::Active((resource_id, handle)) => Self::Active((*resource_id, handle.clone())),
        }
    }
}

impl ReferenceUntyped {
    /// Returns resource id associated with this Reference
    pub fn id(&self) -> ResourceTypeAndId {
        match self {
            Self::Passive(resource_id) => *resource_id,
            Self::Active((resource_id, _handle)) => *resource_id,
        }
    }

    /// Promote a reference to an active handle
    pub fn activate(&mut self, handle: HandleUntyped) {
        if let Self::Passive(resource_id) = self {
            assert_eq!(*resource_id, handle.id());
            *self = Self::Active((*resource_id, handle));
        }
    }

    /// Returns resource id associated with this Reference
    pub fn get_active_handle_untyped(&self) -> Option<&HandleUntyped> {
        if let Self::Active((_resource_id, handle)) = self {
            return Some(handle);
        }
        None
    }

    /// Returns resource id associated with this Reference
    pub fn get_active_handle<T: Resource>(&self) -> Option<Handle<T>> {
        if let Self::Active((_resource_id, handle)) = self {
            return Some(handle.clone().into());
        }
        None
    }
}

impl PartialEq for ReferenceUntyped {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Serialize for ReferenceUntyped {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReferenceUntyped {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // A Reference is always deserialized as passive, and will require activation
        let resource_id = <ResourceTypeAndId>::deserialize(deserializer)?;
        Ok(Self::Passive(resource_id))
    }
}
