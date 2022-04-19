//! Generic data codegen test (offline)

// crate-specific lint exceptions:
// #![allow()]

// generated from def\*.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

pub mod plugin;
pub use plugin::*;
