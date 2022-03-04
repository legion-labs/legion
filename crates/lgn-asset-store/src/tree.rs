use lgn_content_store2::{
    ContentReader, ContentReaderExt, ContentWriter, ContentWriterExt, Identifier,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{Asset, AssetIdentifier, Result};
use std::collections::BTreeMap;

/// A tree of assets.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    children: BTreeMap<String, TreeNode>,
}

/// A tree node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeNode {
    Leaf(AssetIdentifier),
    Branch(TreeIdentifier),
}

/// An identifier for a tree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TreeIdentifier(pub Identifier);

impl Tree {
    /// Create a new tree from the list of its children.
    pub fn new(children: BTreeMap<String, TreeNode>) -> Self {
        Self { children }
    }

    /// Checks whether the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Get the children count.
    pub fn children_count(&self) -> usize {
        self.children.len()
    }

    /// Load a tree from the content-store.
    ///
    /// # Errors
    ///
    /// Returns an error if the tree could not be loaded.
    pub async fn load(
        provider: impl ContentReader + Send + Sync,
        id: &TreeIdentifier,
    ) -> Result<Self> {
        let data = provider.read_content(&id.0).await?;

        Ok(rmp_serde::from_slice(&data)
            .map_err(|err| anyhow::anyhow!("failed to parse tree: {}", err))?)
    }

    /// Save the tree to the content-store.
    ///
    /// # Errors
    ///
    /// Returns an error if the tree could not be saved.
    pub async fn save(&self, provider: impl ContentWriter + Send + Sync) -> Result<TreeIdentifier> {
        let data = self.as_vec();
        let id = provider.write_content(&data).await?;

        Ok(TreeIdentifier(id))
    }

    /// Lookup an asset in the tree.
    ///
    /// If the asset is not found, returns `Ok(None)`.
    /// If the specified key points to a branch, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset exists but could not be loaded.
    pub async fn lookup_asset<Metadata>(
        &self,
        provider: impl ContentReader + Send + Sync,
        key: &str,
    ) -> Result<Option<Asset<Metadata>>>
    where
        Metadata: DeserializeOwned,
    {
        match self.lookup_asset_id(key) {
            Some(id) => Ok(Some(Asset::load(provider, id).await?)),
            None => Ok(None),
        }
    }

    /// Lookup an asset identifier in the tree.
    ///
    /// If the asset is not found, returns `None`.
    /// If the specified key points to a branch, returns `None`.
    pub fn lookup_asset_id(&self, key: &str) -> Option<&AssetIdentifier> {
        if let Some(node) = self.children.get(key) {
            match node {
                TreeNode::Leaf(asset_id) => Some(asset_id),
                TreeNode::Branch(_) => None,
            }
        } else {
            None
        }
    }

    /// Lookup a sub-tree in the tree.
    ///
    /// If the tree is not found, returns `Ok(None)`.
    /// If the specified key points to an asset, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset exists but could not be loaded.
    pub async fn lookup_tree(
        &self,
        provider: impl ContentReader + Send + Sync,
        key: &str,
    ) -> Result<Option<Self>> {
        match self.lookup_tree_id(key) {
            Some(id) => Ok(Some(Self::load(provider, id).await?)),
            None => Ok(None),
        }
    }

    /// Lookup a sub-tree identifier in the tree.
    ///
    /// If the tree is not found, returns `None`.
    /// If the specified key points to an asset, returns `None`.
    pub fn lookup_tree_id(&self, key: &str) -> Option<&TreeIdentifier> {
        if let Some(node) = self.children.get(key) {
            match node {
                TreeNode::Leaf(_) => None,
                TreeNode::Branch(tree_id) => Some(tree_id),
            }
        } else {
            None
        }
    }

    /// Returns an iterator over the children of the tree.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &TreeNode)> {
        self.children.iter().map(|(key, node)| (key.as_str(), node))
    }

    /// Add a named asset to the tree.
    pub fn with_named_asset_id(mut self, key: String, asset_id: AssetIdentifier) -> Self {
        self.children.insert(key, TreeNode::Leaf(asset_id));
        self
    }

    /// Add a named sub-tree to the tree.
    pub fn with_named_tree_id(mut self, key: String, tree_id: TreeIdentifier) -> Self {
        self.children.insert(key, TreeNode::Branch(tree_id));
        self
    }

    /// Remove a child from the tree.
    pub fn without_child(mut self, key: &str) -> Self {
        self.children.remove(key);
        self
    }

    pub fn as_identifier(&self) -> TreeIdentifier {
        TreeIdentifier(Identifier::new(&self.as_vec()))
    }

    fn as_vec(&self) -> Vec<u8> {
        rmp_serde::to_vec(&self).unwrap()
    }
}
