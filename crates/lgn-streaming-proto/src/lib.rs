//! Streaming protocol library.
//!

#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names,
    clippy::return_self_not_must_use
)]

mod streaming {
    tonic::include_proto!("streaming");
}
pub use streaming::*;
