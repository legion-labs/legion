//! Source-control protocol library.
//!

#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names,
    clippy::use_self
)]

mod source_control {
    tonic::include_proto!("source_control");
}
pub use source_control::*;
