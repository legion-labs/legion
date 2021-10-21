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
//! > **TODO**: Describe what steps need to be taken in order to add a new `ResourceType`.

// BEGIN - Legion Labs lints v0.6
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
    clippy::if_not_else,
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
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow(unsafe_code)]
#![warn(missing_docs)]

mod asset_loader;
mod vfs;

mod asset_registry;
pub use asset_registry::*;

mod handle;
pub use handle::*;

mod resource;
pub use resource::*;

mod reference;
pub use reference::Reference;

mod asset;
pub use asset::*;
pub mod manifest;

pub use legion_data_runtime_macros::resource;

#[cfg(test)]
mod test_asset;
