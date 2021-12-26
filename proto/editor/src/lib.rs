#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::wildcard_imports,
    clippy::similar_names
)]

#[path = "../codegen/editor.rs"]
mod editor;
pub use editor::*;
