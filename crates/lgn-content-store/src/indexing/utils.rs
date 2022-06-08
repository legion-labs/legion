use tokio_stream::StreamExt;

use crate::Provider;

use super::{
    errors::{Error, Result},
    BasicIndexer, IndexKey, ResourceIdentifier, Tree, TreeIdentifier, TreeLeafNode, TreeWriter,
};

/// Identifier for an empty tree
///
/// # Errors
///
/// If the tree cannot be written, an error will be returned.
pub async fn empty_tree_id(provider: &Provider) -> Result<TreeIdentifier> {
    provider.write_tree(&Tree::default()).await
}

/// Enumerates all leaves, and collect the resources in the tree-nodes
///
/// # Errors
///
/// If the tree cannot be read, an error will be returned.
pub async fn enumerate_resources(
    provider: &Provider,
    indexer: &(impl BasicIndexer + Sync),
    tree_id: &TreeIdentifier,
) -> Result<Vec<(IndexKey, ResourceIdentifier)>> {
    indexer
        .enumerate_leaves(provider, tree_id)
        .await?
        .map(|(key, leaf_res)| match leaf_res {
            Ok(leaf) => match leaf {
                TreeLeafNode::Resource(resource_id) => Ok((key, resource_id)),
                TreeLeafNode::TreeRoot(_) => Err(Error::CorruptedTree(
                    "found unexpected tree-root node".to_owned(),
                )),
            },
            Err(err) => Err(err),
        })
        .collect::<Result<Vec<(IndexKey, ResourceIdentifier)>>>()
        .await
}
