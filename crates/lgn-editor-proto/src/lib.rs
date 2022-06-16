//! Editor protocol library.
//!

#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names,
    clippy::use_self,
    clippy::return_self_not_must_use
)]
pub mod editor {
    tonic::include_proto!("editor");
}
