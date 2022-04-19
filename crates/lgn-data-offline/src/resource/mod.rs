//! Offline management of resources.
//!
//! [`Project`] keeps track of resources that are part of the project and is
//! responsible for their storage - which includes both on-disk storage and
//! source control interactions.
//!
//! [`ResourceRegistry`] takes responsibility of managing the in-memory
//! representation of resources.

mod project;

pub use self::project::*;

mod metadata;

mod path_name;
pub use self::path_name::*;

mod registry;
pub use self::registry::*;

mod handle;
pub use self::handle::*;

mod resource_handles;
pub use self::resource_handles::*;

mod utils;
pub use self::utils::*;
