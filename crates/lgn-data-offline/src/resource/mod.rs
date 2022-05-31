//! Offline management of resources.
//!
//! [`Project`] keeps track of resources that are part of the project and is
//! responsible for their storage - which includes both on-disk storage and
//! source control interactions.

mod project;

pub use self::project::*;

pub(crate) mod metadata;

mod path_name;
pub use self::path_name::*;

mod resource_handles;
pub use self::resource_handles::*;

mod utils;
pub use self::utils::*;
