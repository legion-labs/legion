//! Indexing facilities backed by the content-store.

mod errors;
mod graphviz_visitor;
mod index;
mod index_key;
mod index_path;
mod indexable_resource;
mod indexer;
mod json_visitor;
mod search_result;
mod static_indexer;
mod string_path_indexer;
mod tree;

pub use errors::{Error, Result};
pub use graphviz_visitor::GraphvizVisitor;
pub use index::{Index, IndexIdentifier, IndexReader, IndexWriter};
pub use index_key::{IndexKey, IndexKeyBound, IndexKeyDisplayFormat};
pub use indexable_resource::{
    IndexableResource, ResourceIdentifier, ResourceReader, ResourceWriter,
};
pub use indexer::{Indexer, IndexerIdentifier, IndexerReader, IndexerWriter};
pub use json_visitor::JsonVisitor;
pub(crate) use search_result::SearchResult;
pub use static_indexer::StaticIndexer;
pub use string_path_indexer::StringPathIndexer;
pub use tree::{
    tree_diff, tree_leaves, tree_visit, Tree, TreeBranchInfo, TreeDiffSide, TreeIdentifier,
    TreeLeafInfo, TreeLeafNode, TreeNode, TreeReader, TreeVisitor, TreeVisitorAction, TreeWriter,
};

pub(crate) use index_path::{IndexPath, IndexPathItem};
