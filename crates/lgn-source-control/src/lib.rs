//! Source control library

// crate-specific lint exceptions:
#![allow(clippy::missing_errors_doc)]

pub mod attach_branch;
pub mod blob_storage_url;
pub mod branch;
pub mod commit;
pub mod config;
pub mod data_types;
pub mod delete;
pub mod detach_branch;
pub mod diff;
mod error;
pub mod import_git_repo;
pub mod index;
pub mod init_workspace;
pub mod local_change;
pub mod local_workspace_connection;
pub mod lock;
pub mod log;
pub mod merge_branch;
pub mod repository_connection;
pub mod resolve;
pub mod revert;
pub mod sql;
pub mod switch_branch;
pub mod sync;
pub mod tree;
mod utils;
pub mod workspace;

pub use crate::log::*;
pub use attach_branch::*;
pub use blob_storage_url::*;
pub use branch::*;
pub use commit::*;
pub use config::*;
pub use data_types::*;
pub use delete::*;
pub use detach_branch::*;
pub use diff::*;
pub use error::*;
pub use import_git_repo::*;
pub use index::*;
pub use init_workspace::*;
pub use local_change::*;
pub use local_workspace_connection::*;
pub use lock::*;
pub use merge_branch::*;
pub use repository_connection::*;
pub use resolve::*;
pub use revert::*;
pub use switch_branch::*;
pub use sync::*;
pub use tree::*;
pub(crate) use utils::*;
pub use workspace::*;
