//! Scripting library - currently using the MUN language

// generated from def\script.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

mod plugin;
pub use plugin::*;
