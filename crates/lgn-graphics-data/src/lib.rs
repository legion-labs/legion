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

pub(crate) mod helpers;

#[cfg(feature = "runtime")]
#[path = "runtime/texture.rs"]
pub mod runtime_texture;

#[cfg(feature = "offline")]
#[path = "offline/png.rs"]
pub mod offline_png;

#[cfg(feature = "offline")]
#[path = "offline/psd.rs"]
pub mod offline_psd;

#[cfg(feature = "offline")]
#[path = "offline/texture.rs"]
pub mod offline_texture;

#[cfg(feature = "offline")]
#[path = "offline/gltf.rs"]
pub mod offline_gltf;
