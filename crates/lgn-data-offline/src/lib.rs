//! Offline resource management module of data processing pipeline.
//!
//! * Tracking Issue: [legion/crate/#37](https://github.com/legion-labs/legion/issues/37)
//! * Design Doc:
//!   [legion/book/project-resources](/book/data-pipeline/project-resources.
//!   html)
//!
//! The module is responsible for management of `resources` - data in offline
//! format, optimized for editing and writing, which is operated on by the
//! editor and various tools.
//!
//! [`resource::Project`] keeps track of resources that are part of the project
//! and is responsible for their storage - which includes both on-disk storage
//! and source control interactions. [`resource::ResourceRegistry`] takes
//! responsibility of managing the in-memory representation of `resources`.
//!
//! From [`resource::Project`]'s perspective there are two kinds of resources:
//! * *local resources* - those modified by local user
//! * *remote resources* - those synced using backing source-control.

// crate-specific lint exceptions:
#![allow(unsafe_code, clippy::missing_errors_doc)]
#![warn(missing_docs)]

pub mod resource;

mod resourcepathid;
pub use resourcepathid::*;
