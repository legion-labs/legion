use crate::{Handle, Resource, ResourceId};
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
