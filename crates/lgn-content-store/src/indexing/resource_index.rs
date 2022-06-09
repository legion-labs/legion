use crate::Provider;

use super::{
    empty_tree_id, BasicIndexer, IndexKey, ResourceIdentifier, Result, TreeIdentifier, TreeLeafNode,
};

pub struct ResourceIndex<Indexer> {
    indexer: Indexer,
    tree_id: TreeIdentifier,
}

impl<Indexer> ResourceIndex<Indexer>
where
    Indexer: BasicIndexer,
{
    pub async fn new(indexer: Indexer, provider: &Provider) -> Self {
        let tree_id = empty_tree_id(provider).await.unwrap();
        Self::new_with_id(indexer, tree_id)
    }

    pub fn new_with_id(indexer: Indexer, tree_id: TreeIdentifier) -> Self {
        Self { indexer, tree_id }
    }

    pub fn id(&self) -> TreeIdentifier {
        self.tree_id.clone()
    }

    /// Add a non-existing leaf to the tree.
    ///
    /// # Errors
    ///
    /// If the leaf at the specified index key already exists, this function
    /// will return `Error::IndexTreeLeafNodeAlreadyExists`.
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    pub async fn add_resource(
        &mut self,
        provider: &Provider,
        index_key: &IndexKey,
        resource_id: ResourceIdentifier,
    ) -> Result<()> {
        self.tree_id = self
            .indexer
            .add_leaf(
                provider,
                &self.tree_id,
                index_key,
                TreeLeafNode::Resource(resource_id),
            )
            .await?;

        Ok(())
    }

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
    pub async fn replace_resource(
        &mut self,
        provider: &Provider,
        index_key: &IndexKey,
        resource_id: ResourceIdentifier,
    ) -> Result<TreeLeafNode> {
        let (tree_id, leaf_node) = self
            .indexer
            .replace_leaf(
                provider,
                &self.tree_id,
                index_key,
                TreeLeafNode::Resource(resource_id),
            )
            .await?;
        self.tree_id = tree_id;

        Ok(leaf_node)
    }

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
    pub async fn remove_resource(
        &mut self,
        provider: &Provider,
        index_key: &IndexKey,
    ) -> Result<TreeLeafNode> {
        let (tree_id, leaf_node) = self
            .indexer
            .remove_leaf(provider, &self.tree_id, index_key)
            .await?;
        self.tree_id = tree_id;

        Ok(leaf_node)
    }
}
