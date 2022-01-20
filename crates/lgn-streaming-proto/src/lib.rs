//! Streaming protocol library.
//!

#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names
)]

#[path = "../codegen/streaming.rs"]
mod streaming;
pub use streaming::*;
