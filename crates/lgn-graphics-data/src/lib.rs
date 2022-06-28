//! Graphics Data Definition

// crate-specific lint exceptions:

//! Graphics

// crate-specific lint exceptions:

//! Generic data codegen test (offline)

// crate-specific lint exceptions:
// #![allow()]

// generated from def\*.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

pub mod color;
pub use color::Color;

pub mod common;
pub use common::*;

/// Plugin module to register support types
pub mod plugin;
pub use plugin::*;

#[cfg(feature = "offline")]
#[path = "offline/png_utils.rs"]
pub mod png_utils;
#[cfg(feature = "offline")]
#[path = "offline/psd_utils.rs"]
pub mod psd_utils;

#[cfg(feature = "offline")]
#[path = "offline/gltf_utils.rs"]
pub mod gltf_utils;
