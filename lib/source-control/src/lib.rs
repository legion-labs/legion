//! Source control library
//!

// BEGIN - Legion Labs lints v0.5
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow(clippy::missing_errors_doc)]

pub mod attach_branch;
pub mod blob_storage;
pub mod blob_storage_spec;
pub mod branch;
pub mod commit;
pub mod config;
pub mod delete;
pub mod destroy_repository;
pub mod detach_branch;
pub mod diff;
pub mod disk_blob_storage;
pub mod forest;
pub mod http_repository_query;
pub mod import_git_repo;
pub mod init_local_repository;
pub mod init_remote_repository;
pub mod init_workspace;
pub mod local_change;
pub mod local_workspace_connection;
pub mod lock;
pub mod log;
pub mod merge_branch;
pub mod ping;
pub mod repository_addr;
pub mod repository_config;
pub mod repository_connection;
pub mod repository_query;
pub mod resolve;
pub mod revert;
pub mod s3_blob_storage;
pub mod server_request;
pub mod sql;
pub mod sql_repository_query;
pub mod switch_branch;
pub mod sync;
pub mod tree;
pub mod utils;
pub mod workspace;

pub use attach_branch::*;
pub use blob_storage::*;
pub use blob_storage_spec::*;
pub use branch::*;
pub use commit::*;
pub use config::*;
pub use delete::*;
pub use detach_branch::*;
pub use diff::*;
pub use disk_blob_storage::*;
pub use forest::*;
pub use import_git_repo::*;
pub use init_local_repository::*;
pub use init_remote_repository::*;
pub use init_workspace::*;
pub use local_change::*;
pub use local_workspace_connection::*;
pub use lock::*;
pub use log::*;
pub use merge_branch::*;
pub use ping::*;
pub use repository_addr::*;
pub use repository_config::*;
pub use repository_connection::*;
pub use repository_query::*;
pub use resolve::*;
pub use revert::*;
pub use s3_blob_storage::*;
pub use server_request::*;
pub use switch_branch::*;
pub use sync::*;
pub use tree::*;
pub use utils::*;
pub use workspace::*;

pub(crate) use telemetry::trace_scope;
