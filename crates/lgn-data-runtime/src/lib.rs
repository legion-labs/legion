//! Runtime asset management module of data processing pipeline.
//!
//! * Tracking Issue: [legion/module/#39](https://github.com/legion-labs/legion/issues/39)
//! * Design Doc:
//!   [legion/book/runtime-assets](/book/data-pipeline/runtime-assets.html)
//!
//! > **WORK IN PROGRESS**: *This document describes the current state of the
//! implementation > and highlights near future development plans.*
//!
//! This module defines runtime asset types.
//!
//! An `asset file` contains one `primary asset` and zero or more `secondary
//! assets`. The path of the asset file is based on the id of the primary asset.
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
//!     - Keep `pointer_fixup_table` until loading of the dependencies to
//!       finishes
//! - For all sections:
//!     - Read section type to choose the `AssetCreator` to use for loading
//!       assets
//!     - Read assets contained in the section
//! - After all dependencies are loaded:
//!     - Process `pointer_fixup_table`
//!     - Trigger load-init on appropriate `AssetCreator`s
//!
//! Different files can be processed differently. Some can read data directly to
//! the destined memory location, other can read data to a temporary one and
//! only store a result of a `load-init` transformation.
//!
//! ## Adding New Asset Type
//!
//! > **TODO**: Describe what steps need to be taken in order to add a new
//! `ResourceType`.

// crate-specific lint exceptions:
#![allow(unsafe_code)]
#![warn(missing_docs)]

mod asset_loader;
mod vfs;
pub use vfs::Device;

mod asset_registry;
pub use asset_registry::*;

mod handle;
pub use handle::*;

mod resource;
pub use resource::*;

mod reference;
pub use reference::ReferenceUntyped;

mod component;
pub use component::*;

mod component_installer;
pub use component_installer::*;

mod resourcepathid;
pub use resourcepathid::*;

mod resource_type;
pub use resource_type::*;

mod resource_id;
pub use resource_id::*;

mod resource_installer;
pub use resource_installer::*;

/// Most commonly used re-exported types.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        AssetRegistry, AssetRegistryError, AssetRegistryMessage, AssetRegistryOptions,
        AssetRegistryReader, Component, ComponentInstaller, EditHandle, EditHandleUntyped, Handle,
        HandleUntyped, LoadRequest, Resource, ResourceDescriptor, ResourceId, ResourceInstaller,
        ResourcePathId, ResourceType, ResourceTypeAndId, Transform,
    };
}
