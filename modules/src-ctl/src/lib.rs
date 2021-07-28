//! Source control library
//!

// BEGIN - Legion Labs lints v0.2
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_crate_level_docs,
    rust_2018_idioms
)]
// END - Legion Labs standard lints v0.2
// crate-specific exceptions:
#![allow()]

pub mod attach_branch;
pub mod blob_storage;
pub mod blob_storage_spec;
pub mod branch;
pub mod commit;
pub mod config;
pub mod delete;
pub mod detach_branch;
pub mod diff;
pub mod disk_blob_storage;
pub mod forest;
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
