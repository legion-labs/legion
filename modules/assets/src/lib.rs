//! Runtime asset management module of data processing pipeline.
//!
//! * Tracking Issue: [legion/module/#39](https://github.com/legion-labs/legion/issues/39)
//! * Design Doc: [legion/book/runtime-assets](/book/data-pipeline/runtime-assets.html)
//!
//! > **WORK IN PROGRESS**: *This document describes the current state of the implementation
//! > and highlights near future development plans.*
//!
//! This module defines runtime asset types.
//!
//! An `asset file` contains one `primary asset` and zero or more `secondary assets`. The path of the asset file is based on the id of the primary asset.
//!
//!
//! ## `Asset File` format
//! ```markdown
//! |--------- header ----------|
//! | magic number, header_size |
//! | list of dependencies      |
//! | pointer_fixup_table       |
//! |-------- section #1 -------|
//! | section_type, asset_count |
//! |---------------------------|
//! | asset #1                  |
//! |---------------------------|
//! | asset #2                  |
//! |-------- section #2 -------|
//! | section_type, asset_count |
//! |---------------------------|
//! | ...
//! > **TODO**: Write short description of the asset file format.
//! ```
//!
//! ## Asset File Loading
//!
//! To read assets from a file (disk, archive, network):
//! - Open the requested file
//! - Read the header into a temporary memory:
//!     - Schedule a load of dependencies
//!     - Keep `pointer_fixup_table` until loading of the dependencies to finishes
//! - For all sections:
//!     - Read section type to choose the `AssetCreator` to use for loading assets
//!     - Read assets contained in the section
//! - After all dependencies are loaded:
//!     - Process `pointer_fixup_table`
//!     - Trigger load-init on appropriate `AssetCreator`s
//!
//! Different files can be processed differently. Some can read data directly to the destined memory location, other can read data to a temporary one and only store a result of a `load-init` transformation.
//!
//! ## Adding New Asset Type
//!
//! > **TODO**: Describe what steps need to be taken in order to add a new `AssetType`.

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

mod assetloader;

mod handle;
pub use handle::*;

mod types;
pub use types::*;

mod assetregistry;
pub use assetregistry::*;

pub mod test_asset;
