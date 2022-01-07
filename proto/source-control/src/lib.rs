//! Source-control protocol library.
//!

#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::wildcard_imports,
    clippy::similar_names
)]

#[path = "../codegen/source_control.rs"]
mod source_control;
pub use source_control::*;
