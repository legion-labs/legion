use lgn_content_store2::{
    ContentReader, ContentReaderExt, ContentWriter, ContentWriterExt, Identifier,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{AssetIdentifier, Result};
use std::collections::{BTreeMap, BTreeSet};

/// A hierarchical tree of assets where the leafs are single assets.
pub type SingleAssetTree = Tree<AssetIdentifier>;

/// A hierarchical tree of assets where the leafs are lists of assets.
pub type MultiAssetsTree = Tree<BTreeSet<AssetIdentifier>>;

/// A tree of assets.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree<LeafType> {
    children: BTreeMap<String, TreeNode<LeafType>>,
}

impl<LeafType: Clone> Clone for Tree<LeafType> {
    fn clone(&self) -> Self {
        Self {
            children: self.children.clone(),
        }
    }
}

impl<LeafType> Default for Tree<LeafType> {
    fn default() -> Self {
        Self {
            children: BTreeMap::default(),
        }
    }
}

/// A tree node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeNode<LeafType> {
    Leaf(LeafType),
    Branch(TreeIdentifier),
}

/// An identifier for a tree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TreeIdentifier(pub Identifier);

impl<LeafType> Tree<LeafType>
where
    LeafType: DeserializeOwned,
{
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

    /// Lookup a sub-tree in the tree.
    ///
    /// If the tree is not found, returns `Ok(None)`.
    /// If the specified key points to an asset, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset exists but could not be loaded.
    pub async fn lookup_branch(
        &self,
        provider: impl ContentReader + Send + Sync,
        key: &str,
    ) -> Result<Option<Self>> {
        match self.lookup_branch_id(key) {
            Some(id) => Ok(Some(Self::load(provider, id).await?)),
            None => Ok(None),
        }
    }
}

impl<LeafType> Tree<LeafType>
where
    LeafType: Serialize,
{
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

    pub fn as_identifier(&self) -> TreeIdentifier {
        TreeIdentifier(Identifier::new(&self.as_vec()))
    }

    fn as_vec(&self) -> Vec<u8> {
        rmp_serde::to_vec(&self).unwrap()
    }
}

impl<LeafType> Tree<LeafType> {
    /// Create a new tree from the list of its children.
    pub fn new(children: BTreeMap<String, TreeNode<LeafType>>) -> Self {
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

    /// Lookup a node in the tree.
    ///
    /// If the leaf is not found, returns `None`.
    /// If the specified key points to a branch, returns `None`.
    pub fn lookup(&self, key: &str) -> Option<&TreeNode<LeafType>> {
        self.children.get(key)
    }

    /// Lookup a leaf in the tree.
    ///
    /// If the leaf is not found, returns `None`.
    /// If the specified key points to a branch, returns `None`.
    pub fn lookup_leaf(&self, key: &str) -> Option<&LeafType> {
        if let Some(node) = self.children.get(key) {
            match node {
                TreeNode::Leaf(leaf) => Some(leaf),
                TreeNode::Branch(_) => None,
            }
        } else {
            None
        }
    }

    /// Lookup a sub-tree identifier in the tree.
    ///
    /// If the tree is not found, returns `None`.
    /// If the specified key points to a leaf, returns `None`.
    pub fn lookup_branch_id(&self, key: &str) -> Option<&TreeIdentifier> {
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
    pub fn iter(&self) -> impl Iterator<Item = (&str, &TreeNode<LeafType>)> {
        self.children.iter().map(|(key, node)| (key.as_str(), node))
    }

    /// Add a named leaf to the tree.
    pub fn with_named_leaf(mut self, key: String, leaf: LeafType) -> Self {
        self.children.insert(key, TreeNode::Leaf(leaf));
        self
    }

    /// Add a named sub-tree to the tree.
    pub fn with_named_branch(mut self, key: String, branch: TreeIdentifier) -> Self {
        self.children.insert(key, TreeNode::Branch(branch));
        self
    }

    /// Remove a child from the tree.
    pub fn without_child(mut self, key: &str) -> Self {
        self.children.remove(key);
        self
    }
}
