//! Data build module of data processing pipeline.
//!
//! > **WORK IN PROGRESS**: *This document describes the current state of the
//! implementation > and highlights near future development plans.*
//!
//! Data build module is part of the engine's data processing pipeline. Its main
//! responsibility is to transform the resources from the `offline format` used
//! by the editor into a `runtime format` which is consumed by the engine.
//!
//! Runtime data generation is handled by a collection of `data compilers` -
//! limited in scope modules dedicated to processing a given type of *input
//! resource and its dependencies*. One such process can produce **many**
//! outputs in form of **derived resources** - either as a supporting
//! *intermediate* data format or data format ready to be consumed by the
//! runtime engine.
//!
//! Data compilation is:
//! - **Hermetic** - dependent only on a known set of inputs.
//! - **Deterministic** - the result is bit-by-bit reproducible given the same
//!   set of inputs.
//!
//! All the results of data-compilation are stored in a
//! [`ContentStore`](`lgn_content_store::ContentStore`) and a manifest
//! file containing the metadata about the results is returned.
//!
//! To support incremental building the data build is persisted on disk in
//! `source.index` and `output.index` files. Those files contain:
//! - **source.index** - The build-oriented data structure describing resources
//!   and build dependencies in the [`project`] that is being built.
//! - **output.index** - Records of derived resources that are stored in a
//!   [`ContentStore`](`lgn_content_store::ContentStore`).
//!
//! For other parts of the data pipeline see [`lgn_data_offline`],
//! [`lgn_data_runtime`] and [`lgn_data_compiler`] modules.
//!
//! # Structure on disk
//!
//! An example of a [`project`] with 1 source file, 2 offline resources and 2
//! derived resources on disk looks as follows. (where **temp/** is an build
//! output directory acting as a *local content store*)
//!
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
//!  |   |- source.index
//!  |   |- output.index
//! ```
//!
//! # Data Build Process
//!
//! The build process consists of the following steps:
//!
//! 1. An update of the `build index` with changes found in the corresponding
//! [`project`]. 2. Processing of data build input arguments:
//!     - Find and validate the **source resource** in the [`project`].
//!     - Validating **build input parameters**: `platform`, `target`,
//!       `environment`, `locale`.
//! 3. Building a **build graph** from input **compile path** and all its
//! dependencies. 4. Gather information about all required **data compilers**.
//!     - The information includes **Compiler Hash** based on **build input
//!       parameters**
//! 5. Process **build graph** nodes (*source* and *target* tuples) in order of
//! dependencies:
//!     - Compute **Context Hash** using **Compiler Hash** and **Databuild
//!       Version**
//!     - Compute **Source Hash** in 2 ways depending on the *source build graph
//!       node*:
//!         - when it is a **source resource**: use a hash of the checksum of
//!           its content and all content of its dependencies.
//!         - when it is a **derived resource**: use the checksum of the output
//!           of it's *source build graph node*.
//! 6. Check the `build index` if there is already existing output for given
//! (**Context Hash**, **Source Hash**) tuple. 7. If not, compile the resource:
//!     - Store the resulting resource in
//!       [`ContentStore`](`lgn_content_store::ContentStore`) and a record an
//!       entry in `build index`.
//!     - Add the compiled resource to the resulting `manifest file`.
//!
//! # `SourceHash` and `ContextHash`
//!
//! The role of the two ids is two allow for incremental data compilation. They
//! are the signature of the resource and the signature of the context for which
//! they are compiled for. Both values are used in `build index` to cache the
//! compilation results and to be able to retrieve the results in consecutive
//! builds. Both are created from number of sources:
//!
//! #### `SourceHash` - the signature of the compilation source data
//!
//! It identifies the content of a resource being compiled. It is defined in two
//! ways, depending on the resource it describes:
//!
//! * For **source resource**:
//!     * checksum of the resource's content (available in [`.meta`] file).
//!     * checksum of content of each of the resource's dependencies (list of
//!       dependencies is in [`.meta`] file)
//! * For **derived resource**:
//!     * checksum of the output of the directly dependent data compilation (as
//!       described in the
//!       [`ResourcePathId`](`lgn_data_offline::ResourcePathId`))
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
//! > **TODO**: The above does not take into account `feature switches` that
//! would give  more granular control on the behavior of the data compiler.
//!
//! [`.meta`]: ../resources/index.html#resource-meta-file
//! [`project`]: ../resources/index.html#project-index

// crate-specific lint exceptions:
#![allow(unsafe_code, clippy::missing_errors_doc)]
#![warn(missing_docs)]

use std::path::PathBuf;

use lgn_data_compiler::compiler_api::CompilerError;
use thiserror::Error;

/// Data build error. todo(kstasik): revisit how errors are handled/propagated
#[derive(Error, Debug)]
pub enum Error {
    /// Project-related error.
    #[error("Project-related error: '{0}")]
    Project(#[from] lgn_data_offline::resource::Error),

    /// Not found.
    #[error("Not found.")]
    NotFound,
    /// Compiler not found.
    #[error("Compiler not found.")]
    CompilerNotFound,
    /// IO error.
    #[error("IO error.")]
    Io(Box<dyn std::error::Error + Send + Sync>),
    /// Circular dependency in build graph.
    #[error("Circular dependency in build graph.")]
    CircularDependency,
    /// Index version mismatch.
    #[error("Index version mismatch: '{value}', expected: '{expected}'")]
    VersionMismatch {
        /// Current version value.
        value: String,
        /// Expected version value.
        expected: String,
    },
    /// Content Store invalid.
    #[error("Content Store invalid.")]
    InvalidContentStore,
    /// Project invalid.
    #[error("Project invalid.")]
    InvalidProject(PathBuf),
    /// Manifest file error.
    #[error("Manifest file error.")]
    InvalidManifest(Box<dyn std::error::Error + Send + Sync>),
    /// Asset linking failed.
    #[error("Asset linking failed.")]
    LinkFailed,
    /// Compilation did not produce expected output.
    #[error("Compilation did not produce expected output.")]
    OutputNotPresent,
    /// Compiler returned an error.
    #[error("Compiler returned an error: '{0}'")]
    Compiler(#[source] CompilerError),
    /// Source Index error.
    #[error("Source Index error")]
    SourceIndex,
}

mod asset_file_writer;
mod databuild;
mod options;
mod output_index;
mod source_index;

pub use databuild::*;
pub use options::*;
