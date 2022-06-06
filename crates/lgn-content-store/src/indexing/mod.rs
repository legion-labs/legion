//! Indexing facilities backed by the content-store.

mod composite_indexer;
mod errors;
mod graphviz_visitor;
mod index_key;
mod index_path;
mod indexable_resource;
mod json_visitor;
mod search_result;
mod static_indexer;
mod string_path_indexer;
mod tree;
mod utils;

use std::{ops::RangeBounds, pin::Pin};

use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use tokio_stream::StreamExt;

pub use composite_indexer::CompositeIndexer;
pub use errors::{Error, Result};
pub use graphviz_visitor::GraphvizVisitor;
pub use index_key::{IndexKey, IndexKeyBound, IndexKeyDisplayFormat};
pub(crate) use index_path::{IndexPath, IndexPathItem};
pub use indexable_resource::{
    IndexableResource, ResourceExists, ResourceIdentifier, ResourceReader, ResourceWriter,
};
pub use json_visitor::JsonVisitor;
pub(crate) use search_result::SearchResult;
pub use static_indexer::StaticIndexer;
pub use string_path_indexer::StringPathIndexer;
pub(crate) use tree::tree_leaves;
pub use tree::{
    tree_diff, SharedTreeIdentifier, Tree, TreeBranchInfo, TreeDiffSide, TreeIdentifier,
    TreeLeafInfo, TreeLeafNode, TreeNode, TreeReader, TreeVisitor, TreeVisitorAction, TreeWriter,
};
pub use utils::enumerate_resources;

use crate::Provider;

/// BasicIndexer implements the basic method that all indexers should implement.
///
/// A basic indexer can search for leaves by exact path, and add, replace and
/// remove them.
///
/// Index keys in a basic indexer have no required concept of ordering, although
/// specific implementation can chose to enforce this internally, as an
/// optimization measure.
///
/// In that case, the indexer probably implements additional traits that take
/// advantage of those specific constraints.
#[async_trait]
pub trait BasicIndexer {
    /// Get a leaf node from the tree.
    ///
    /// This function will return `None` if the tree does not contain a leaf
    /// with the specified key.
    ///
    /// # Errors
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    async fn get_leaf(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &IndexKey,
    ) -> Result<Option<TreeLeafNode>>;

    /// Add a non-existing leaf to the tree.
    ///
    /// # Errors
    ///
    /// If the leaf at the specified index key already exists, this function
    /// will return `Error::IndexTreeLeafNodeAlreadyExists`.
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    async fn add_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
        leaf_node: TreeLeafNode,
    ) -> Result<TreeIdentifier>;

    /// Replace an existing leaf in the tree.
    ///
    /// # Returns
    ///
    /// The new tree and the old leaf replaced by the new one.
    ///
    /// # Cost
    ///
    /// Replacing a leaf is generally fast as no tree rebalancing can occur.
    ///
    /// # Errors
    ///
    /// If the leaf at the specified index key does not exist, this function
    /// will return `Error::IndexTreeLeafNodeNotFound`.
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    async fn replace_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
        leaf_node: TreeLeafNode,
    ) -> Result<(TreeIdentifier, TreeLeafNode)>;

    /// Remove an existing leaf from the tree.
    ///
    /// # Returns
    ///
    /// The new tree and the old removed leaf.
    ///
    /// # Errors
    ///
    /// If the leaf at the specified index key does not exist, this function
    /// will return `Error::IndexTreeLeafNodeNotFound`.
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    async fn remove_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
    ) -> Result<(TreeIdentifier, TreeLeafNode)>;

    /// Enumerate all the leaves.
    ///
    /// # Warning
    ///
    /// Calling this on a big tree can be very slow.
    ///
    /// # Errors
    ///
    /// If the tree cannot be read, an error will be returned.
    async fn enumerate_leaves<'s>(
        &'s self,
        provider: &'s Provider,
        root_id: &'s TreeIdentifier,
    ) -> Result<Pin<Box<dyn Stream<Item = (IndexKey, Result<TreeLeafNode>)> + Send + 's>>> {
        Ok(Box::pin(tree_leaves(
            provider,
            root_id,
            IndexKey::default(),
        )))
    }
}

#[async_trait]
pub trait OrderedIndexer {
    /// Returns a stream that iterates over all leaves in the specified tree
    /// that belong to the specified range.
    ///
    /// # Warning
    ///
    /// This method may iterate over the entire tree. If used on a real, large
    /// tree it could actually take a very long time to end. Think twice before
    /// using it with a large range.
    ///
    /// # Errors
    ///
    /// If the range is invalid, an error will be returned.
    async fn enumerate_leaves_in_range<'s, T, R>(
        &'s self,
        provider: &'s Provider,
        root_id: &'s TreeIdentifier,
        range: R,
    ) -> Result<Pin<Box<dyn Stream<Item = (IndexKey, Result<TreeLeafNode>)> + Send + 's>>>
    where
        T: Into<IndexKey> + Clone,
        R: RangeBounds<T> + Send + 's;
}

/// A recursive indexer builds trees whose branches are fully-valid sub-trees
/// (for instance: filesystems).
///
/// As such, those indexer have the ability to perform queries with partial
/// index keys and possibly return branches.
///
/// They can also list all the leaves in such a branch.
#[async_trait]
pub trait RecursiveIndexer {
    /// Get a leaf or branch node from the tree.
    ///
    /// This function will return `None` if the tree does not contain a leaf or
    /// branch with the specified key.
    ///
    /// # Errors
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    async fn get(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &IndexKey,
    ) -> Result<Option<TreeNode>>;

    /// Enumerate all the leaves in a specific branch.
    ///
    /// If the index key points to a leaf, only that leaf will be returned.
    ///
    /// Otherwise, all the leaves in the branch will be returned.
    ///
    /// # Errors
    ///
    /// If the specific index key does not point to a leaf or branch,
    /// `IndexTreeNodeNotFound` is returned.
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    async fn enumerate_leaves_at<'s>(
        &'s self,
        provider: &'s Provider,
        root_id: &'s TreeIdentifier,
        index_key: &'s IndexKey,
    ) -> Result<Pin<Box<dyn Stream<Item = (IndexKey, Result<TreeLeafNode>)> + Send + 's>>> {
        match self.get(provider, root_id, index_key).await? {
            Some(node) => Ok(match node {
                TreeNode::Leaf(leaf_node) => Box::pin(futures::stream::once(async {
                    (index_key.clone(), Ok(leaf_node))
                })),
                TreeNode::Branch(branch_node) => Box::pin(stream! {
                    let leaves = tree_leaves(provider, &branch_node, index_key.clone());

                    tokio::pin!(leaves);

                    while let Some(res) = leaves.next().await {
                        yield res;
                    }
                }),
            }),
            None => Err(Error::IndexTreeNodeNotFound(index_key.clone())),
        }
    }
}
