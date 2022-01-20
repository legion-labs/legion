//! Editor protocol library.
//!

#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names
)]

#[path = "../codegen/editor.rs"]
mod editor;
pub use editor::*;

#[path = "../codegen/resource_browser.rs"]
pub mod resource_browser;

#[path = "../codegen/property_inspector.rs"]
pub mod property_inspector;

#[path = "../codegen/scene_explorer.rs"]
pub mod scene_explorer;
