use async_trait::async_trait;
use serde::{de::Visitor, Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::{Identifier, Provider};

use super::{Error, IndexKey, IntoIndexKey, ResourceIdentifier, Result};

/// Represents a tree identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TreeIdentifier(pub(crate) Identifier);

impl TreeIdentifier {
    pub(crate) fn as_identifier(&self) -> &Identifier {
        &self.0
    }

    /// Visit all nodes in the tree, from root to leaves.
    ///
    /// The order of visit is not guaranteed, but it is guaranteed that parent
    /// nodes will be fully visited before their children.
    ///
    /// As a direct consequence, the first visited node will be the root node.
    ///
    /// # Errors
    ///
    /// If the tree is corrupted, an error will be returned.
    ///
    /// If the visitor returns an error, the iteration will stop and the error
    /// will be returned.
    pub async fn visit<Visitor: TreeVisitor + Send>(
        &self,
        provider: &Provider,
        mut visitor: Visitor,
    ) -> Result<Visitor> {
        let root = provider.read_tree(self).await?;

        if visitor.visit_root(self, &root).await? == TreeVisitorAction::Continue {
            let mut stack: VecDeque<_> = vec![(self.clone(), root, IndexKey::default())].into();

            while let Some(parent) = stack.pop_front() {
                for (local_key, node) in parent.1.children {
                    let key = parent.2.join(&local_key);

                    match node {
                        TreeNode::Branch(branch_id) => {
                            let branch = provider.read_tree(&branch_id).await?;

                            if visitor
                                .visit_branch(
                                    &parent.0,
                                    &key,
                                    &local_key,
                                    &branch_id,
                                    &branch,
                                    stack.len(),
                                )
                                .await?
                                == TreeVisitorAction::Continue
                            {
                                stack.push_back((branch_id.clone(), branch, key));
                            }
                        }
                        TreeNode::Leaf(leaf_node) => {
                            visitor
                                .visit_leaf(&parent.0, &key, &local_key, &leaf_node, stack.len())
                                .await?;
                        }
                    }
                }
            }
        }

        visitor.visit_done(self).await?;

        Ok(visitor)
    }
}

impl Display for TreeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TreeIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.parse() {
            Ok(id) => Ok(Self(id)),
            Err(err) => Err(Error::InvalidTreeIdentifier(err)),
        }
    }
}

/// A tree node type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeNode {
    /// A leaf node that points to a leaf which can be another index tree root
    /// or a resource.
    Leaf(TreeLeafNode),
    /// A branch node that points to a sub-tree within a given index.
    Branch(TreeIdentifier),
}

impl Display for TreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeNode::Leaf(leaf) => write!(f, "{}", leaf),
            TreeNode::Branch(branch) => write!(f, "{}", branch),
        }
    }
}

impl TreeNode {
    pub fn as_identifier(&self) -> &Identifier {
        match self {
            TreeNode::Leaf(leaf) => leaf.as_identifier(),
            TreeNode::Branch(branch) => branch.as_identifier(),
        }
    }

    pub fn into_leaf(self) -> Option<TreeLeafNode> {
        match self {
            TreeNode::Leaf(leaf) => Some(leaf),
            TreeNode::Branch(_) => None,
        }
    }

    pub fn into_branch(self) -> Option<TreeIdentifier> {
        match self {
            TreeNode::Branch(id) => Some(id),
            TreeNode::Leaf(_) => None,
        }
    }

    pub fn as_leaf(&self) -> Option<&TreeLeafNode> {
        match self {
            TreeNode::Leaf(leaf) => Some(leaf),
            TreeNode::Branch(_) => None,
        }
    }

    pub fn as_branch(&self) -> Option<&TreeIdentifier> {
        match self {
            TreeNode::Branch(id) => Some(id),
            TreeNode::Leaf(_) => None,
        }
    }
}

// All this below ensures that `TreeNode` takes up the minimum amount of space
// when serialized.
const TREE_NODE_LEAF_RESOURCE: u8 = 0;
const TREE_NODE_LEAF_TREE_ROOT: u8 = 1;
const TREE_NODE_BRANCH: u8 = 2;

impl Serialize for TreeNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TreeNode::Leaf(TreeLeafNode::Resource(id)) => {
                (TREE_NODE_LEAF_RESOURCE, id).serialize(serializer)
            }
            TreeNode::Leaf(TreeLeafNode::TreeRoot(id)) => {
                (TREE_NODE_LEAF_TREE_ROOT, id).serialize(serializer)
            }
            TreeNode::Branch(id) => (TREE_NODE_BRANCH, id).serialize(serializer),
        }
    }
}
struct TreeNodeVisitorImpl;

impl<'de> Visitor<'de> for TreeNodeVisitorImpl {
    type Value = TreeNode;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "a tuple (node-type, identifier)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let node_type = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_length(0, &"a tuple (node-type, identifier)")
        })?;

        match node_type {
            TREE_NODE_LEAF_RESOURCE => {
                let id = seq.next_element()?.ok_or_else(|| {
                    serde::de::Error::invalid_length(1, &"a tuple (node-type, identifier)")
                })?;

                Ok(TreeNode::Leaf(TreeLeafNode::Resource(id)))
            }
            TREE_NODE_LEAF_TREE_ROOT => {
                let id = seq.next_element()?.ok_or_else(|| {
                    serde::de::Error::invalid_length(1, &"a tuple (node-type, identifier)")
                })?;

                Ok(TreeNode::Leaf(TreeLeafNode::TreeRoot(id)))
            }
            TREE_NODE_BRANCH => {
                let id = seq.next_element()?.ok_or_else(|| {
                    serde::de::Error::invalid_length(1, &"a tuple (node-type, identifier)")
                })?;

                Ok(TreeNode::Branch(id))
            }
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(node_type.into()),
                &"a node type",
            )),
        }
    }
}

impl<'de> Deserialize<'de> for TreeNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, TreeNodeVisitorImpl)
    }
}

/// A tree node type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeLeafNode {
    /// A resource.
    Resource(ResourceIdentifier),
    /// Another index tree root.
    TreeRoot(TreeIdentifier),
}

impl Display for TreeLeafNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeLeafNode::Resource(id) => write!(f, "{}", id),
            TreeLeafNode::TreeRoot(id) => write!(f, "{}", id),
        }
    }
}

impl TreeLeafNode {
    pub(crate) fn as_identifier(&self) -> &Identifier {
        match self {
            TreeLeafNode::Resource(resource) => resource.as_identifier(),
            TreeLeafNode::TreeRoot(tree_identifier) => tree_identifier.as_identifier(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TreeVisitorAction {
    Continue,
    Stop,
}

#[async_trait]
pub trait TreeVisitor {
    async fn visit_root(
        &mut self,
        _root_id: &TreeIdentifier,
        _root: &Tree,
    ) -> Result<TreeVisitorAction> {
        Ok(TreeVisitorAction::Continue)
    }

    async fn visit_branch(
        &mut self,
        _parent_id: &TreeIdentifier,
        _key: &IndexKey,
        _local_key: &IndexKey,
        _branch_id: &TreeIdentifier,
        _branch: &Tree,
        _depth: usize,
    ) -> Result<TreeVisitorAction> {
        Ok(TreeVisitorAction::Continue)
    }

    async fn visit_leaf(
        &mut self,
        _parent_id: &TreeIdentifier,
        _key: &IndexKey,
        _local_key: &IndexKey,
        _leaf: &TreeLeafNode,
        _depth: usize,
    ) -> Result<()> {
        Ok(())
    }

    async fn visit_done(&mut self, _root_id: &TreeIdentifier) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    #[serde(rename = "c", default)]
    pub(crate) count: usize,
    #[serde(rename = "s", default)]
    pub(crate) total_size: usize,
    // The direct children of this tree.
    //
    // You may wonder why we don't use a `BTreeMap` or `HashMap` here: we need
    // to actually control and maintain the ordering of the elements. A
    // `BTreeMap` would be fine, but it doesn't implement binary searches and
    // its memory representation is not cache friendly, which makes reading
    // needlessly slow.
    //
    // As we expect thousands of elements per `Tree` and our first goal is to
    // make index reads as fast as possible, we use a `Vec` and implement this
    // ourselves. We may revisit this in the future though.
    #[serde(rename = "n", default)]
    pub(crate) children: Vec<(IndexKey, TreeNode)>,
}

impl Tree {
    fn as_vec(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }

    fn from_slice(buf: &[u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(buf)?)
    }

    pub(crate) fn into_children(mut self, key: &[u8]) -> Option<TreeNode> {
        match self
            .children
            .binary_search_by_key(&key, |(k, _)| k.as_slice())
        {
            Ok(idx) => Some(self.children.swap_remove(idx).1),
            Err(_) => None,
        }
    }

    /// Insert a children into the tree with the specified key.
    ///
    /// If a children already exists with the specified key, it will be replaced
    /// and the previous value will be returned.
    ///
    /// Otherwise, `None` will be returned.
    pub(crate) fn insert_children(&mut self, key: &[u8], node: TreeNode) -> Option<TreeNode> {
        match self
            .children
            .binary_search_by(|(k, _)| k.as_slice().cmp(key))
        {
            Ok(idx) => {
                Some(std::mem::replace(&mut self.children[idx], (key.into_index_key(), node)).1)
            }
            Err(idx) => {
                self.children.insert(idx, (key.into_index_key(), node));

                None
            }
        }
    }

    /// Remove a children from the tree with the specified key.
    ///
    /// If a children exists with the specified key, it will be removed and
    /// returned.
    ///
    /// Otherwise, `None` will be returned and the tree remains unchanged.
    pub(crate) fn remove_children(&mut self, key: &[u8]) -> Option<TreeNode> {
        match self
            .children
            .binary_search_by(|(k, _)| k.as_slice().cmp(key))
        {
            Ok(idx) => Some(self.children.remove(idx).1),
            Err(_) => None,
        }
    }

    /// The direct number of children.
    pub fn direct_count(&self) -> usize {
        self.children.len()
    }

    /// The total number of nodes in the tree.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The total size of all the leaves in the tree.
    ///
    /// Only available if the associated indexer has been configured to
    /// calculate the size.
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Returns whether the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

#[async_trait]
pub trait TreeReader {
    async fn read_tree(&self, id: &TreeIdentifier) -> Result<Tree>;
}

#[async_trait]
impl TreeReader for Provider {
    async fn read_tree(&self, id: &TreeIdentifier) -> Result<Tree> {
        let buf = self.read(&id.0).await?;

        Tree::from_slice(&buf)
    }
}

#[async_trait]
pub trait TreeWriter {
    async fn write_tree(&self, tree: &Tree) -> Result<TreeIdentifier>;
}

#[async_trait]
impl TreeWriter for Provider {
    async fn write_tree(&self, tree: &Tree) -> Result<TreeIdentifier> {
        let buf = tree.as_vec();

        self.write(&buf)
            .await
            .map(TreeIdentifier)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_properties() {
        let tree = Tree::default();

        assert_eq!(tree.count(), 0);
        assert_eq!(tree.total_size(), 0);
    }

    #[test]
    fn test_tree_node_serialization() {
        let id = Identifier::new_data(&[1, 2, 3]);
        let tree_node = TreeNode::Branch(TreeIdentifier(id.clone()));
        let buf = rmp_serde::to_vec(&tree_node).unwrap();
        let res = rmp_serde::from_slice(&buf).unwrap();
        assert_eq!(tree_node, res);

        let tree_node = TreeNode::Leaf(TreeLeafNode::Resource(ResourceIdentifier(id.clone())));
        let buf = rmp_serde::to_vec(&tree_node).unwrap();
        let res = rmp_serde::from_slice(&buf).unwrap();
        assert_eq!(tree_node, res);

        let tree_node = TreeNode::Leaf(TreeLeafNode::TreeRoot(TreeIdentifier(id)));
        let buf = rmp_serde::to_vec(&tree_node).unwrap();
        let res = rmp_serde::from_slice(&buf).unwrap();
        assert_eq!(tree_node, res);
    }

    #[test]
    fn test_tree_serialization() {
        let tree = Tree::default();
        let buf = tree.as_vec();

        // See [MsgPack
        // spec](https://github.com/msgpack/msgpack/blob/master/spec.md#formats)
        // for details.
        assert_eq!(&buf, &[0x93, 0x00, 0x00, 0x90]);

        let t1 = Tree::from_slice(&buf).unwrap();
        assert_eq!(t1, tree);

        let tree = Tree {
            count: 1,
            total_size: 3,
            children: vec![(
                IndexKey::from_slice(b"a"),
                TreeNode::Leaf(TreeLeafNode::Resource(ResourceIdentifier(
                    Identifier::new_data(b"foo"),
                ))),
            )],
        };
        let buf = tree.as_vec();

        // See [MsgPack
        // spec](https://github.com/msgpack/msgpack/blob/master/spec.md#formats)
        // for details.
        assert_eq!(
            &buf,
            &[
                0x93, 0x01, 0x03, 0x91, 0x92, 0x91, 0x61, 0x92, 0x00, 0xC4, 0x04, 0x00, 0x66, 0x6F,
                0x6F
            ]
        );

        let t1 = Tree::from_slice(&buf).unwrap();
        assert_eq!(t1, tree);
    }
}
