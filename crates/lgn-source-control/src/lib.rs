//! Source control library

// crate-specific lint exceptions:
#![allow(clippy::missing_errors_doc)]

pub mod attach_branch;
pub mod blob_storage_url;
pub mod branch;
pub mod config;
pub mod data_types;
pub mod detach_branch;
//pub mod diff;
mod error;
//pub mod import_git_repo;
pub mod index;
pub mod lock;
//pub mod merge_branch;
//pub mod resolve;
//pub mod revert;
pub mod sql;
//pub mod switch_branch;
//pub mod sync;
mod utils;
pub mod workspace;

pub use attach_branch::*;
pub use blob_storage_url::*;
pub use branch::*;
pub use config::*;
pub use data_types::*;
pub use detach_branch::*;
//pub use diff::*;
pub use error::*;
//pub use import_git_repo::*;
pub use index::*;
pub use lock::*;
//pub use merge_branch::*;
//pub use resolve::*;
//pub use revert::*;
//pub use switch_branch::*;
//pub use sync::*;
pub(crate) use utils::*;
pub use workspace::*;
