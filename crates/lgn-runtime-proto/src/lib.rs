//! Runtime protocol library.
//!

#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names
)]
mod runtime {
    tonic::include_proto!("runtime");
}
