//! Scripting library - currently has an integration for the MUN language, Rune and Rhai.

// generated from def\script.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

mod labels;
pub use labels::*;

mod plugin;
pub use plugin::*;
