//! Offline management of resources.
//!
//! [`Project`] keeps track of resources that are part of the project and is
//! responsible for their storage - which includes both on-disk storage and
//! source control interactions.

// crate-specific lint exceptions:
#![allow(unsafe_code, clippy::missing_errors_doc, missing_docs, dead_code)]

// generated from def\*.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

mod metadata;
pub use self::metadata::*;

mod project;
pub use self::project::*;

mod resource_path_name;
pub use self::resource_path_name::*;

mod source_resource;
pub use self::source_resource::*;

mod json_utils;
pub use self::json_utils::*;

pub mod vfs;
