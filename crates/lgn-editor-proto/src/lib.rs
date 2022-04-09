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

pub mod resource_browser {
    tonic::include_proto!("resource_browser");
}

pub mod property_inspector {
    tonic::include_proto!("property_inspector");
}

pub mod source_control {
    tonic::include_proto!("source_control");
}
