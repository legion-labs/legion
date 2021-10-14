use crate::{AssetRegistry, Handle, Resource, ResourceId};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::any::Any;

/// A `ResourceReference` represents a reference to an external resource, that can be promoted to a handle
#[derive(Serialize, Deserialize)]
pub enum Reference<T>
where
    T: Any + Resource,
{
    /// Reference is not yet active, and is simply described as an id
    Passive(ResourceId),

    /// Reference is unset
    None,

    /// Reference is active, and be accessed through a typed handle
    #[serde(skip)]
    Active(Handle<T>),
}

impl<T> Reference<T>
where
    T: Any + Resource,
{
    /// Promote a reference to an active handle
    ///
    /// # Errors
    ///
    /// Will return an error if attempting to activate a reference to a resource that
    /// has not already been loaded.
    pub fn activate(&mut self, registry: &mut AssetRegistry) -> Result<()> {
        if let Self::Passive(resource_id) = self {
            if let Some(handle) = registry.get_untyped(*resource_id) {
                println!("activating reference to resource {}", resource_id);
                *self = Self::Active(handle.into());
            } else {
                eprintln!("failed to activate reference to resource {}", resource_id);
                return Err(Error::msg(
                    "activating a reference to a resource that has not been loaded",
                ));
            }
        }
        Ok(())
    }
}
