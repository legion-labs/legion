//! Offline management of resources.
//!

mod project;
pub use self::project::*;

mod metadata;
pub use self::metadata::*;

mod types;
pub use self::types::*;

mod registry;
pub use self::registry::*;

#[cfg(test)]
pub(crate) mod test_resource;
