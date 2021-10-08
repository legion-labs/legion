//! Offline resource management module of data processing pipeline.
//!
//! * Tracking Issue: [legion/crate/#37](https://github.com/legion-labs/legion/issues/37)
//! * Design Doc: [legion/book/project-resources](/book/data-pipeline/project-resources.html)
//!
//! The module is responsible for management of `resources` - data in offline format, optimized for editing and writing,
//! which is operated on by the editor and various tools.
//!
//! [`resource::Project`] keeps track of resources that are part of the project and is responsible for their storage - which includes
//! both on-disk storage and source control interactions. [`resource::ResourceRegistry`] takes responsibility
//! of managing the in-memory representation of `resources`.
//!
//! From [`resource::Project`]'s perspective there are two kinds of resources:
//! * *local resources* - those modified by local user
//! * *remote resources* - those synced using backing source-control.

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
#![allow(unsafe_code, clippy::missing_errors_doc)]
#![warn(missing_docs)]

pub mod data_container;
pub mod resource;

mod resourcepathid;
pub use resourcepathid::*;
