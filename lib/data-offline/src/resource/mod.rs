//! Offline management of resources.
//!
//! [`Project`] keeps track of resources that are part of the project and is responsible for their storage - which includes both on-disk storage and source control interactions.
//!
//! [`ResourceRegistry`] takes responsibility of managing the in-memory representation of resources.

pub use legion_data_offline_macros::Resource;

/// Types implementing `Resource` represent editor data.
pub trait Resource: 'static {}

/// Note: Based on impl of dyn Any
impl dyn Resource {
    /// Returns `true` if the boxed type is the same as `T`.
    /// (See [`std::any::Any::is`](https://doc.rust-lang.org/std/any/trait.Any.html#method.is))
    #[inline]
    pub fn is<T: Resource>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_ref`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref))
    #[inline]
    pub fn downcast_ref<T: Resource>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*((self as *const dyn Resource).cast::<T>())) }
        } else {
            None
        }
    }

    /// Returns some mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_mut`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_mut))
    #[inline]
    pub fn downcast_mut<T: Resource>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *((self as *mut dyn Resource).cast::<T>())) }
        } else {
            None
        }
    }
}

/// The `ResourceProcessor` trait allows to process an offline resource.
pub trait ResourceProcessor {
    /// Interface returning a resource in a default state. Useful when creating a new resource.
    fn new_resource(&mut self) -> Box<dyn Resource>;

    /// Interface returning a list of resources that `resource` depends on for building.
    fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<AssetPathId>;

    /// Interface defining serialization behavior of the resource.
    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn io::Write,
    ) -> io::Result<usize>;

    /// Interface defining deserialization behavior of the resource.
    fn read_resource(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Resource>>;
}

mod project;
use std::any::Any;
use std::any::TypeId;
use std::io;

use crate::asset::AssetPathId;

pub use self::project::*;

mod metadata;
pub use self::metadata::*;

mod types;
pub use self::types::*;

mod registry;
pub use self::registry::*;

mod handle;
pub use self::handle::*;

#[cfg(test)]
pub(crate) mod test_resource;
