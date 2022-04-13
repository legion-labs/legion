//! Runtime protocol library.
//!

#![allow(
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::return_self_not_must_use,
    clippy::similar_names,
    clippy::use_self,
    clippy::wildcard_imports
)]

pub mod log_stream {
    tonic::include_proto!("log_stream");
}
