//! Type Reflection crate

// crate-specific lint exceptions:
#![allow(unsafe_code, clippy::missing_errors_doc)]
#![warn(missing_docs)]

mod base_descriptor;
pub use base_descriptor::*;

mod primitive_descriptor;
pub use primitive_descriptor::*;

mod type_reflection;
pub use type_reflection::*;

mod box_dyn_descriptor;
pub use box_dyn_descriptor::*;

mod array_descriptor;
pub use array_descriptor::*;

mod option_descriptor;
pub use option_descriptor::*;

mod struct_descriptor;
pub use struct_descriptor::*;

mod field_descriptor;
pub use field_descriptor::*;

/// Utilities to serializing reflection using `bincode`
pub mod bincode_utils;
/// Utilities to collecting reflection
pub mod collector;
/// Utilities to serializing reflection using `serde_json`
pub mod json_utils;
/// Utilities to serializing reflection
pub mod utils;
