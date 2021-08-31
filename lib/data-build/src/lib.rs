//! Data build module of data processing pipeline.
//!
//! > **WORK IN PROGRESS**: *This document describes the current state of the implementation
//! > and highlights near future development plans.*
//!
//! Data build module is part of the engine's data processing pipeline. Its main responsibility
//! is to transform the resources from the `offline format` used by the editor
//! into a `runtime format` which is consumed by the engine.
//!
//! Runtime data generation is handled by a collection of `data compilers` - limited in scope modules
//! dedicated to processing a given type of *input resource and its dependencies*. One such process
//! can produce **many** outputs in form of **derived resources** - either as a supporting *intermediate* data format
//! or data format ready to be consumed by the runtime engine.
//!
//! Data compilation is:
//! - **Hermetic** - dependent only on a known set of inputs.
//! - **Deterministic** - the result is bit-by-bit reproducible given the same set of inputs.
//!
//! All the results of data-compilation are stored in a [`ContentStore`](`legion_content_store::ContentStore`) and a manifest
//! file containing the metadata about the results is returned.
//!
//! To support incremental building the data build is persisted in a file on disk. This file is called `build.index`.
//! It contains:
//! - The build-oriented data structure describing resources and build dependencies in the [`project`] that is being built.
//! - Records of compiled assets that are stored in a [`ContentStore`](`legion_content_store::ContentStore`).
//!
//! For other parts of the data pipeline see [`legion_data_offline`], [`legion_data_runtime`] and [`legion_data_compiler`] modules.
//!
//! # Structure on disk
//!
//! An example of a [`project`] with 1 source file, 2 offline resources and 2 compiled assets on disk looks as follows.
//! (where **temp/** is an build output directory acting as a *local compiled content store*)
//! ```markdown
//!  ./
//!  | + source/
//!  | |- foo.psd
//!  | |- foo.export
//!  | + offline/
//!  | |- a81fb4498cd04368
//!  | |- a81fb4498cd04368.meta
//!  | |- 8063daaf864780d6
//!  | |- 8063daaf864780d6.meta
//!  | + temp/
//!  |   |- 561fd98d4d97405a
//!  |   |- a00438b586a19d4f
//!  |   |- build.index
//!  | - project.index
//! ```
//!
//! # Data Build Process
//!
//! The build process consists of the following steps:
//!
//! 1. An update of the `build index` with changes found in the corresponding [`project`].
//! 2. Processing of data build input arguments:
//!     - Find and validate the **source resource** in the [`project`].
//!     - Validating **build input parameters**: `platform`, `target`, `environment`, `locale`.
//! 3. Building a **build graph** from input **compile path** and all its dependencies.
//! 4. Gather information about all required **data compilers**.
//!     - The information includes **Compiler Hash** based on **build input parameters**
//! 5. Process **build graph** nodes (*source* and *target* tuples) in order of dependencies:
//!     - Compute **Context Hash** using **Compiler Hash** and **Databuild Version**
//!     - Compute **Source Hash** in 2 ways depending on the *source build graph node*:
//!         - when it is a **source resource**: use a hash of the checksum of its content and all content of its dependencies.
//!         - when it is a **derived resource**: use the checksum of the output of it's *source build graph node*.
//! 6. Check the `build index` if there is already existing output for given (**Context Hash**, **Source Hash**) tuple.
//! 7. If not, compile the resource:
//!     - Store the resulting resource in [`ContentStore`](`legion_content_store::ContentStore`) and a record an entry in `build index`.
//!     - Add the compiled resource to the resulting `manifest file`.
//!
//! # `SourceHash` and `ContextHash`
//!
//! The role of the two ids is two allow for incremental data compilation. They are the signature of the resource and the signature of the
//! context for which they are compiled for. Both values are used in `build index` to cache the compilation results and to be able to retrieve
//! the results in consecutive builds. Both are created from number of sources:
//!
//! #### `SourceHash` - the signature of the compilation source data
//!
//! It identifies the content of a resource being compiled. It is defined in two ways, depending on the resource it describes:
//!
//! * For **source resource**:
//!     * checksum of the resource's content (available in [`.meta`] file).
//!     * checksum of content of each of the resource's dependencies (list of dependencies is in [`.meta`] file)
//! * For **derived resource**:
//!     * checksum of the output of the directly dependent data compilation (as described in the [`AssetPathId`](`legion_data_offline::asset::AssetPathId`))
//!
//! #### `ContextHash` - the signature of the compilation context
//!
//! It identifies the context of compilation of the `SourceHash` resource:
//! * Type of the resource compiled
//! * `CompilerHash` - a compiler-defined value based on:
//!   * Compiler code version.
//!   * Compiler data format version.
//!   * Compilation target - i.e.: Game, Server, etc.
//!   * Compilation platform - i.e.: Linux, Windows, Console, etc.
//!   * Compilation locale - i.e.: Language, Region, etc.
//! * Data-build process version.
//!
//! > **TODO**: The above does not take into account `feature switches` that would give  more granular control on the behavior of the data compiler.
//!
//! [`.meta`]: ../resources/index.html#resource-meta-file
//! [`project`]: ../resources/index.html#project-index

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

#[derive(Debug)]
/// Data build error. todo(kstasik): revisit how errors are handled/propagated
pub enum Error {
    /// Project-related error
    ProjectError,
    /// Not found.
    NotFound,
    /// Compiler not found.
    CompilerNotFound,
    /// IO error.
    IOError,
    /// Index integrity error.
    IntegrityFailure,
    /// Circular dependency in build graph.
    CircularDependency,
    /// Index version mismatch.
    VersionMismatch,
    /// Compiled Asset Store invalid.
    InvalidAssetStore,
    /// Project invalid.
    InvalidProject,
    /// Manifest file error.
    InvalidManifest,
    /// Asset linking failed.
    LinkFailed,
    /// Compiler returned an error.
    CompilerError(io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::ProjectError => write!(f, "ResourceMgmntError"),
            Error::NotFound => write!(f, "NotFound"),
            Error::CompilerNotFound => write!(f, "CompilerNotFound"),
            Error::IOError => write!(f, "IOError"),
            Error::IntegrityFailure => write!(f, "IntegrityFailure"),
            Error::CircularDependency => write!(f, "CircularDependency"),
            Error::VersionMismatch => write!(f, "VersionMismatch"),
            Error::InvalidAssetStore => write!(f, "InvalidCompiledAssetStore"),
            Error::InvalidProject => write!(f, "InvalidProject"),
            Error::InvalidManifest => write!(f, "InvalidManifest"),
            Error::LinkFailed => write!(f, "LinkFailed"),
            Error::CompilerError(_) => write!(f, "CompilerError"),
        }
    }
}

impl From<legion_data_offline::resource::Error> for Error {
    fn from(err: legion_data_offline::resource::Error) -> Self {
        match err {
            legion_data_offline::resource::Error::NotFound
            | legion_data_offline::resource::Error::InvalidPath => Self::NotFound,
            legion_data_offline::resource::Error::ParseError
            | legion_data_offline::resource::Error::IOError(_) => Self::ProjectError,
        }
    }
}

mod asset_file_writer;
mod buildindex;
mod databuild;
mod options;

use std::io;

pub use databuild::*;
pub use options::*;
