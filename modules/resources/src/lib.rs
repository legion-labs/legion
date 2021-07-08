//! Offline resource management module.
//!
//! > **WORK IN PROGRESS**: *This document describes the current state of the implementation
//! > and highlights near future development plans.*
//!
//! The module is responsible for keeping track of offline resources.
//! This includes `local resources` - those modified by the local user and
//! `remote resources` - those synced using the backing source-control.
//!
//! Resource modification includes change of resource's content, name or location as well as adding or removing a resource.
//!
//! # Project Index
//!
//! The state of the project is read from a file once [`Project`] is opened and kept in memory throughout its lifetime.
//! The changes are written back to the file once [`Project`] is dropped.
//!
//! The state of the project consists of two sets of [`ResourceId`]s:
//! - Local [`ResourceId`] list - locally modified resources.
//! - Remote [`ResourceId`] list - synced resources.
//!
//! A resource consists of a resource content file and a `.meta` file associated to it.
//! [`ResourceId`] is enough to locate a resource content file and its associated `.meta` file on disk.
//!
//! An example of a project with 2 offline resources on disk looks as follows:
//! ```markdown
//! ./
//!  |- project.index
//!  |- a81fb4498cd04368
//!  |- a81fb4498cd04368.meta
//!  |- 8063daaf864780d6
//!  |- 8063daaf864780d6.meta
//! ```
//!
//! ## Resource `.meta` file
//!
//! The information in the `.meta` file includes:
//! - List of [`ResourceId`]s of resource's build dependencies.
//! - Resource's name/path.
//! - Md5 checksum of resource's content file.
//!
//! Note: Resource's name/path is only used for display purposes and can be changed freely.
//!
//! # Future Work
//!
//! - [x] Change resource name to a path.
//! - [ ] Use directory path to open a project.
//! - [ ] Make changes to resource content be reflected in the .meta file (`content_md5`).
//! - [ ] Use `SQLite` to persist project's state.
//! - [ ] Use source-control to retrieve remote work and commit local edits.
//! - [ ] Cache resource names to avoid opening .meta files
//! - [ ] Cache resource dependencies to avoid opening .meta files.
//! - [ ] Cache invalidation when editing resources.
//! - [ ] On-commit re-calculation of .meta dependencies (i.e. root level when an object in other file is added/removed).
//! - [ ] Resource deletion.
//! - [ ] Revisit ResourcePath - currently is OS-dependent (PathBuf)
//! - [ ] Safe generation of ResourceId (avoid clashes)

// BEGIN - Legion Labs lints v0.2
// do not change or add/remove here, but one can add exceptions after this section
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
#![warn(missing_docs)]

mod project;
pub use self::project::*;

mod metadata;
pub use self::metadata::*;

mod types;
pub use self::types::*;
