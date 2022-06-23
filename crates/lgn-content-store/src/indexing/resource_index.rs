use std::sync::Arc;

use tokio_stream::StreamExt;

use crate::Provider;

use super::{
    empty_tree_id, AsIndexKey, BasicIndexer, Error, IndexKey, ResourceIdentifier, Result,
    SharedTreeIdentifier, TreeIdentifier, TreeLeafNode,
};

enum TreeIdentifierType {
    Exclusive(TreeIdentifier),
    Shared(SharedTreeIdentifier),
}

pub struct ResourceIndex<T>
where
    T: AsIndexKey,
{
    provider: Arc<Provider>,
    indexer: <T as AsIndexKey>::Indexer,
    tree_id: TreeIdentifierType, // to do: maybe generic, supporting TreeIdentifierAccessor trait (implemented for TreeIdentifier and SharedTreeIdentifier)
}

impl<T> ResourceIndex<T>
where
    T: AsIndexKey,
{
    pub async fn new_exclusive(provider: Arc<Provider>) -> Self {
        let tree_id = empty_tree_id(&provider).await.unwrap();
        Self::new_exclusive_with_id(provider, tree_id)
    }

    pub fn new_exclusive_with_id(provider: Arc<Provider>, tree_id: TreeIdentifier) -> Self {
        Self {
            provider,
            indexer: <T as AsIndexKey>::new_indexer(),
            tree_id: TreeIdentifierType::Exclusive(tree_id),
        }
    }

    pub async fn new_shared(provider: Arc<Provider>) -> Self {
        let tree_id = empty_tree_id(&provider).await.unwrap();
        Self::new_shared_with_raw_id(provider, tree_id)
    }

    pub fn new_shared_with_raw_id(provider: Arc<Provider>, tree_id: TreeIdentifier) -> Self {
        Self::new_shared_with_id(provider, SharedTreeIdentifier::new(tree_id))
    }

    pub fn new_shared_with_id(
        provider: Arc<Provider>,
        shared_tree_id: SharedTreeIdentifier,
    ) -> Self {
        Self {
            provider,
            indexer: <T as AsIndexKey>::new_indexer(),
            tree_id: TreeIdentifierType::Shared(shared_tree_id),
        }
    }

    pub fn id(&self) -> TreeIdentifier {
        match &self.tree_id {
            TreeIdentifierType::Exclusive(tree_id) => tree_id.clone(),
            TreeIdentifierType::Shared(tree_id) => tree_id.read(),
        }
    }

    pub fn provider(&self) -> &Arc<Provider> {
        &self.provider
    }

    pub fn indexer(&self) -> &<T as AsIndexKey>::Indexer {
        &self.indexer
    }

    pub fn shared_id(&self) -> SharedTreeIdentifier {
        match &self.tree_id {
            TreeIdentifierType::Exclusive(tree_id) => SharedTreeIdentifier::new(tree_id.clone()),
            TreeIdentifierType::Shared(tree_id) => tree_id.clone(),
        }
    }

    pub fn set_id(&mut self, id: TreeIdentifier) {
        match &mut self.tree_id {
            TreeIdentifierType::Exclusive(tree_id) => {
                *tree_id = id;
            }
            TreeIdentifierType::Shared(tree_id) => tree_id.write(id),
        }
    }

    /// Get a leaf node from the tree.
    ///
    /// This function will return `None` if the tree does not contain a leaf
    /// with the specified key.
    ///
    /// # Errors
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    pub async fn get_identifier(&self, index_key: T) -> Result<Option<ResourceIdentifier>> {
        let index_key: IndexKey = index_key.into();
        let leaf_node = self
            .indexer
            .get_leaf(&self.provider, &self.id(), &index_key)
            .await?;

        match leaf_node {
            Some(leaf_node) => match leaf_node {
                TreeLeafNode::Resource(resource_id) => Ok(Some(resource_id)),
                TreeLeafNode::TreeRoot(_tree_id) => {
                    Err(Error::CorruptedTree("expected resource node".to_owned()))
                }
            },
            None => Ok(None),
        }
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
        index_key: T,
        resource_id: ResourceIdentifier,
    ) -> Result<()> {
        let index_key: IndexKey = index_key.into();
        let tree_id = self
            .indexer
            .add_leaf(
                &self.provider,
                &self.id(),
                &index_key,
                TreeLeafNode::Resource(resource_id),
            )
            .await?;
        self.set_id(tree_id);

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
        index_key: T,
        resource_id: ResourceIdentifier,
    ) -> Result<ResourceIdentifier> {
        let index_key: IndexKey = index_key.into();
        let (tree_id, leaf_node) = self
            .indexer
            .replace_leaf(
                &self.provider,
                &self.id(),
                &index_key,
                TreeLeafNode::Resource(resource_id),
            )
            .await?;
        self.set_id(tree_id);

        match leaf_node {
            TreeLeafNode::Resource(resource_id) => Ok(resource_id),
            TreeLeafNode::TreeRoot(_) => Err(Error::CorruptedTree(
                "found unexpected tree-root node".to_owned(),
            )),
        }
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
    pub async fn remove_resource(&mut self, index_key: T) -> Result<ResourceIdentifier> {
        let index_key: IndexKey = index_key.into();
        let (tree_id, leaf_node) = self
            .indexer
            .remove_leaf(&self.provider, &self.id(), &index_key)
            .await?;
        self.set_id(tree_id);

        match leaf_node {
            TreeLeafNode::Resource(resource_id) => Ok(resource_id),
            TreeLeafNode::TreeRoot(_) => Err(Error::CorruptedTree(
                "found unexpected tree-root node".to_owned(),
            )),
        }
    }

    /// Enumerate all the leaves.
    ///
    /// # Warning
    ///
    /// Calling this on a big tree can be very slow.
    ///
    /// # Errors
    ///
    /// If the tree cannot be read, an error will be returned.
    pub async fn enumerate_resources(&self) -> Result<Vec<(T, ResourceIdentifier)>> {
        self.indexer
            .enumerate_leaves(&self.provider, &self.id())
            .await?
            .map(|(key, leaf_res)| match leaf_res {
                Ok(leaf) => match leaf {
                    TreeLeafNode::Resource(resource_id) => Ok((key.into(), resource_id)),
                    TreeLeafNode::TreeRoot(_) => Err(Error::CorruptedTree(
                        "found unexpected tree-root node".to_owned(),
                    )),
                },
                Err(err) => Err(err),
            })
            .collect::<Result<Vec<(T, ResourceIdentifier)>>>()
            .await
    }
}
