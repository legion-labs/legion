//! Indexing facilities backed by the content-store.

mod composite_index_key;
mod errors;
mod filesystem_indexer;
mod graphviz_visitor;
mod index;
mod index_key;
mod index_path;
mod indexable_resource;
mod indexer;
mod json_visitor;
mod search_result;
mod static_indexer;
mod tree;

pub use composite_index_key::CompositeIndexKey;
pub use errors::{Error, Result};
pub use filesystem_indexer::FilesystemIndexer;
pub use graphviz_visitor::GraphvizVisitor;
pub use index::{Index, IndexIdentifier, IndexReader, IndexWriter};
pub use index_key::{IndexKey, IndexKeyBound, IntoIndexKey};
pub use indexable_resource::{
    IndexableResource, ResourceIdentifier, ResourceReader, ResourceWriter,
};
pub use indexer::{Indexer, IndexerIdentifier, IndexerReader, IndexerWriter};
pub use json_visitor::JsonVisitor;
pub(crate) use search_result::SearchResult;
pub use static_indexer::StaticIndexer;
pub use tree::{
    Tree, TreeIdentifier, TreeLeafNode, TreeNode, TreeReader, TreeVisitor, TreeVisitorAction,
    TreeWriter,
};

pub(crate) use index_path::{IndexPath, IndexPathItem};
