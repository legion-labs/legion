//! Data build module of data processing pipeline.
//!
//! > **WORK IN PROGRESS**: *This document describes the current state of the implementation
//! > and highlights near future development plans.*
//!
//! Data build module is part of the engine's data processing pipeline. Its main responsibility
//! is to transform the resources from the `offline format` used by the editor
//! into a `runtime format` which is consumed by the engine.
//!
//! Runtime data generation is handled by a set of `data-compilers` - limited in scope modules
//! dedicated to processing a given type of `input-resource` and its dependencies. One such process
//! can output **many** runtime assets.
//!
//! Data compilation is:
//! - **Hermetic** - dependent only on a known set of inputs.
//! - **Deterministic** - the result is bit-by-bit reproducible given the same set of inputs.
//!
//! All the results of data-compilation are stored in a [`CompiledAssetStore`] and a manifest
//! file containing the metadata about the results is returned.
//!
//! To support incremental building the data build is persisted in a file on disk. This file is called `build.index`.
//! The `build.index` contains:
//! - The build-oriented data structure describing resources and build dependencies in the [`project`] that is being built.
//! - Records of compiled assets that are stored in a [`CompiledAssetStore`].
//!
//! For other parts of the data pipeline see [`legion_resources`] and [`legion_assets`] modules.
//!
//! # Structure on disk
//!
//! An example of a [`project`] with 2 offline resources (including assiciated [`.meta`] files) and 2 compiled assets on disk looks as follows:
//! ```markdown
//! ./
//!  |- offline-data/
//!  | |- project.index
//!  | |- build.index
//!  | |- a81fb4498cd04368
//!  | |- a81fb4498cd04368.meta
//!  | |- 8063daaf864780d6
//!  | |- 8063daaf864780d6.meta
//!  |- asset_store/
//!  |  |- 561fd98d4d97405a
//!  |  |- a00438b586a19d4f
//! ```
//!
//! # Data Build Process
//!
//! The build process consists of the following steps:
//!
//! 1. An update of the `build-index` with recent changes found in the build corresponding [`project`].
//! 2. Processing of data build input arguments:
//!     - Searching for the `input resource` in the [`project`].
//!     - Validating build input parameters: `platform`, `target`, `environment`, `locale`.
//! 3. Retrieving a `data-compiler` for the requested resource type.
//! 4. Listing the `compiler inputs` and `resource inputs`:
//!     - Compiler Inputs:
//!         - Resource Type - type of the compiled resource.
//!         - Compiler Id - the id returned by the data-compiler based on the build input parameters.
//!         - Data Build Version - version of the data build process.
//!     - Resource Inputs:
//!         - Resource Id - id of the compiled resource.
//!         - Resource Hash - hash of the resource's content and the content of its dependencies.
//! 5. Check the `build-index` if there is already existing output for given `(Compiler Input, Resource Input)`.
//! 6. If not, compile the resource:
//!     - store the resulting resource in [`CompiledAssetStore`] and a record in `build-index`.
//!     - add the compiled resource to the resulting `manifest file`.
//!
//! # Future Work
//! - [ ] Creation of DAG of dependencies to be able to process them in the right order.
//! - [ ] Be able to distribute the resource compilation. (build-index does not support concurrent inserts)
//! - [ ] Load dependencies - gather during compilation, use at runtime.
//! - [ ] More flexible compiler registration (compilers-as-executables?).
//! - [ ] Allow one resource to be processed by many compilers (currently only one supported).
//! - [ ] Make `source_hash` include hashes of filtered resource dependencies.
//! - [x] Clarify how the manifest works and how it gets update if a leaf resource is compiled.
//! - [ ] Index compiled assets in two ways: current branch & branchless.
//!
//! [`.meta`]: ../resources/index.html#resource-meta-file
//! [`project`]: ../resources/index.html#project-index

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
#![warn(missing_docs)]

#[derive(Debug, PartialEq)]
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
    /// Index version mismatch.
    VersionMismatch,
    /// Project invalid.
    InvalidProject,
    /// Manifest file error.
    InvalidManifest,
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
            Error::VersionMismatch => write!(f, "VersionMismatch"),
            Error::InvalidProject => write!(f, "InvalidProject"),
            Error::InvalidManifest => write!(f, "InvalidManifest"),
        }
    }
}

impl From<legion_resources::Error> for Error {
    fn from(err: legion_resources::Error) -> Self {
        match err {
            legion_resources::Error::NotFound | legion_resources::Error::InvalidPath => {
                Self::NotFound
            }
            legion_resources::Error::ParseError | legion_resources::Error::IOError(_) => {
                Self::ProjectError
            }
        }
    }
}

/// Build target enumeration.
///
/// `TODO`: This needs to be more extensible.
#[derive(Clone, Copy)]
pub enum Target {
    /// Game client.
    Game,
    /// Server.
    Server,
    /// Backend service.
    Backend,
}

/// Build platform enumeration.
#[derive(Clone, Copy)]
pub enum Platform {
    /// Windows
    Windows,
    /// Linux
    Linux,
    /// Game Console X
    ConsoleX,
}

/// Defines user's language/region.
pub type Locale = [char; 2]; // todo(kstasik): this type is cumbersome in use

mod buildindex;
mod compiledassetstore;
mod compilers;
mod databuild;

pub use self::compiledassetstore::*;
pub use self::databuild::*;
pub use legion_resources::ResourcePath;
