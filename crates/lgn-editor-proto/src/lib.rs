//! Editor protocol library.
//!

#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names
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

pub mod scene_explorer {
    tonic::include_proto!("scene_explorer");
}
