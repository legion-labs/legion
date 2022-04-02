//! Source control library

// crate-specific lint exceptions:
#![allow(clippy::missing_errors_doc)]

pub mod data_types;
//pub mod diff;
mod error;
//pub mod import_git_repo;
pub mod index;
//pub mod merge_branch;
//pub mod resolve;
//pub mod revert;
mod config;
pub mod sql;
mod utils;
pub mod workspace;

pub use data_types::*;
//pub use diff::*;
pub use error::*;
//pub use import_git_repo::*;
pub use index::*;
//pub use merge_branch::*;
//pub use resolve::*;
//pub use revert::*;
pub use config::Config;
pub(crate) use utils::*;
pub use workspace::*;
