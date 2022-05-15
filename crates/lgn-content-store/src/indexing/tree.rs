use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use serde::{de::Visitor, Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    str::FromStr,
};
use tokio_stream::StreamExt;

use crate::{Identifier, Provider};

use super::{Error, IndexKey, ResourceIdentifier, Result};

/// Represents a tree identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TreeIdentifier(pub(crate) Identifier);

impl TreeIdentifier {
    pub(crate) fn as_identifier(&self) -> &Identifier {
        &self.0
    }
}

/// Returns a stream that iterates over all leaves in the tree.
///
/// # Warning
///
/// This method will iterate over the entire tree. If used on a real, large
/// tree it could actually take a very long time to end. Think twice before
/// using it.
pub fn tree_leaves<'s>(
    provider: &'s Provider,
    tree_id: &'s TreeIdentifier,
    base_key: IndexKey,
) -> impl Stream<Item = (IndexKey, Result<TreeLeafNode>)> + 's {
    let mut trees = VecDeque::new();

    stream! {
        let root = match provider.read_tree(tree_id).await {
            Ok(root) => root,
            Err(err) => {
                yield (base_key, Err(err));
                return;
            }
        };
        trees.push_back((base_key, root));

        while let Some((prefix, current_tree)) = trees.pop_front() {
            for (key, node) in current_tree.children {
                let new_prefix = prefix.join(key);

                match node {
                    TreeNode::Leaf(entry) => {
                        yield (new_prefix, Ok(entry));
                    },
                    TreeNode::Branch(id) => {
                        match provider.read_tree(&id).await {
                            Ok(tree) => {
                                trees.push_back((new_prefix, tree));
                            },
                            Err(err) => {
                                yield (new_prefix, Err(err));
                            },
                        };
                    },
                }
            }
        }
    }
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
pub async fn tree_visit<Visitor: TreeVisitor + Send>(
    provider: &Provider,
    tree_id: &TreeIdentifier,
    mut visitor: Visitor,
) -> Result<Visitor> {
    let root = provider.read_tree(tree_id).await?;

    if visitor.visit_root(tree_id, &root).await? == TreeVisitorAction::Continue {
        let mut stack: VecDeque<_> = vec![(tree_id.clone(), root, IndexKey::default())].into();

        while let Some(parent) = stack.pop_front() {
            for (local_key, node) in parent.1.children {
                let parent_id = &parent.0;
                let local_key = &local_key;
                let key = parent.2.join(local_key);

                match node {
                    TreeNode::Branch(branch_id) => {
                        let branch_id = &branch_id;
                        let branch = provider.read_tree(branch_id).await?;

                        if visitor
                            .visit_branch(TreeBranchInfo {
                                parent_id,
                                key: &key,
                                local_key,
                                branch_id,
                                branch: &branch,
                            })
                            .await?
                            == TreeVisitorAction::Continue
                        {
                            stack.push_back((branch_id.clone(), branch, key));
                        }
                    }
                    TreeNode::Leaf(leaf_node) => {
                        let leaf_node = &leaf_node;

                        visitor
                            .visit_leaf(TreeLeafInfo {
                                parent_id,
                                key: &key,
                                local_key,
                                leaf_node,
                            })
                            .await?;
                    }
                }
            }
        }
    }

    visitor.visit_done(tree_id).await?;

    Ok(visitor)
}

/// Compare two trees for differences.
///
/// Identical children are silently ignored. This is in essence, the main
/// optimization that avoids running through the whole tree when comparing
/// identical branches.
///
/// # Returns
///
/// A stream of differences, in an unspecified - but stable - order.
pub fn tree_diff<'s>(
    provider: &'s Provider,
    base_key: &'s IndexKey,
    left_id: &'s TreeIdentifier,
    right_id: &'s TreeIdentifier,
) -> impl Stream<Item = (TreeDiffSide, IndexKey, Result<TreeLeafNode>)> + 's {
    stream! {
        let mut stack: DiffStack = vec![((
            base_key.clone(),
            left_id.clone(),
        ), (
            base_key.clone(),
            right_id.clone(),
        ))].into();

        while let Some(((left_key, left_id), (right_key, right_id))) = stack.pop_front() {
            if (left_key == right_key) {
                let leaves = tree_diff_isopath(provider, left_key, &left_id, &right_id, &mut stack);

                tokio::pin!(leaves);

                while let Some(res) = leaves.next().await {
                    yield res;
                }
            } else if left_id == right_id {
                // Interesting optimization: the content of both nodes
                // is the same but they have different keys!
                //
                // They can't be anything else than different.
                let leaves = tree_leaves(provider, &left_id, left_key);

                tokio::pin!(leaves);

                while let Some((key, node)) = leaves.next().await {
                    yield (TreeDiffSide::Left, key, node);
                }

                let leaves = tree_leaves(provider, &right_id, right_key);

                tokio::pin!(leaves);

                while let Some((key, node)) = leaves.next().await {
                    yield (TreeDiffSide::Left, key, node);
                }
            } else if left_key.len() < right_key.len() {
                let left = match provider.read_tree(&left_id).await {
                    Ok(tree) => tree,
                    Err(err) => {
                        yield (TreeDiffSide::Left, left_key, Err(err));
                        return;
                    }
                };

                for (key, node) in left.children {
                    let key = left_key.join(&key);

                    match node {
                        TreeNode::Branch(left_branch_id) => {
                            // If if the new key and the other one are a
                            // prefix of one another, add the new
                            // candidate pair to the stack.
                            //
                            // If not, abandon the current candidate pair.
                            if key.has_prefix(&right_key) || right_key.has_prefix(&key) {
                                stack.push_back((
                                    (key, left_branch_id),
                                    (right_key.clone(), right_id.clone()),
                                ));
                            }
                        }
                        TreeNode::Leaf(left_leaf_node) => {
                            // The left is a leaf, so we can iterate over
                            // all the leaves from right and yield the ones that
                            // are not equal to left. If none is found that
                            // is equal to left, we must also yield the
                            // left leaf.
                            let leaves = tree_leaves(provider, &right_id, right_key.clone());

                            tokio::pin!(leaves);

                            let mut found = false;

                            while let Some((k, node)) = leaves.next().await {
                                if k != key {
                                    yield (TreeDiffSide::Right, k, node);
                                } else {
                                    found = true;
                                }
                            }

                            if !found {
                                yield (TreeDiffSide::Left, key, Ok(left_leaf_node.clone()));
                            }
                        }
                    }
                }
            } else {
                let right = match provider.read_tree(&right_id).await {
                    Ok(tree) => tree,
                    Err(err) => {
                        yield (TreeDiffSide::Right, right_key, Err(err));
                        return;
                    }
                };

                for (key, node) in right.children {
                    let key = right_key.join(&key);

                    match node {
                        TreeNode::Branch(right_branch_id) => {
                            // If if the new key and the other one are a
                            // prefix of one another, add the new
                            // candidate pair to the stack.
                            //
                            // If not, abandon the current candidate pair.
                            if key.has_prefix(&left_key) || left_key.has_prefix(&key) {
                                stack.push_back((
                                    (left_key.clone(), left_id.clone()),
                                    (key, right_branch_id),
                                ));
                            }
                        }
                        TreeNode::Leaf(right_leaf_node) => {
                            // The right is a leaf, so we can iterate over
                            // all the leaves from left and yield the ones that
                            // are not equal to right. If none is found that
                            // is equal to right, we must also yield the
                            // right leaf.
                            let leaves = tree_leaves(provider, &left_id, left_key.clone());

                            tokio::pin!(leaves);

                            let mut found = false;

                            while let Some((k, node)) = leaves.next().await {
                                if k != key {
                                    yield (TreeDiffSide::Left, k, node);
                                } else {
                                    found = true;
                                }
                            }

                            if !found {
                                yield (TreeDiffSide::Right, key, Ok(right_leaf_node.clone()));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Compare two trees that have the same index path for differences.
fn tree_diff_isopath<'s>(
    provider: &'s Provider,
    base_key: IndexKey,
    left_id: &'s TreeIdentifier,
    right_id: &'s TreeIdentifier,
    stack: &'s mut DiffStack,
) -> impl Stream<Item = (TreeDiffSide, IndexKey, Result<TreeLeafNode>)> + 's {
    stream! {
        // If the keys are identical, we compare left and right in a
        // special way.
        if left_id == right_id {
            return;
        }

        let left = match provider.read_tree(left_id).await {
            Ok(tree) => tree,
            Err(err) => {
                yield (TreeDiffSide::Left, base_key, Err(err));
                return;
            }
        };
        let right = match provider.read_tree(right_id).await {
            Ok(tree) => tree,
            Err(err) => {
                yield (TreeDiffSide::Right, base_key, Err(err));
                return;
            }
        };

        let mut left_children = left.children.as_slice();
        let mut right_children = right.children.as_slice();

        // We loop until either of the slices becomes empty.
        loop {
            let (left_item, right_item) = match (left_children.first(), right_children.first()) {
                (Some(left_item), Some(right_item)) => (left_item, right_item),
                _ => break,
            };

            if left_item.0 == right_item.0 {
                left_children = &left_children[1..];
                right_children = &right_children[1..];

                // If the two nodes have the same key, we need to compare their values.
                if left_item.1 == right_item.1 {
                    // The two nodes are strictly identical, so we can skip them. Yay.
                    continue;
                }

                let key = base_key.join(&left_item.0);

                match (&left_item.1, &right_item.1) {
                    (TreeNode::Branch(left_branch_id), TreeNode::Branch(right_branch_id)) => {
                        // The two branches have the exact same key, so we can
                        // compare them using `diff`: let's add them to the stack.
                        stack.push_back(((key.clone(), left_branch_id.clone()), (key, right_branch_id.clone())));
                    }
                    (TreeNode::Leaf(left_leaf_node), TreeNode::Leaf(right_leaf_node)) => {
                        // The two leaves are different (or we wouldn't be here), so we return them.
                        yield (TreeDiffSide::Left, key.clone(), Ok(left_leaf_node.clone()));
                        yield (TreeDiffSide::Right, key, Ok(right_leaf_node.clone()));
                    }
                    (TreeNode::Branch(left_branch_id), TreeNode::Leaf(right_leaf_node)) => {
                        //We can return the right leaf right away: we also
                        //need to visit the left branch fully and return all its
                        //leaves.
                        yield (TreeDiffSide::Right, key.clone(), Ok(right_leaf_node.clone()));

                        let leaves = tree_leaves(provider, left_branch_id, key);

                        tokio::pin!(leaves);

                        while let Some((key, node)) = leaves.next().await {
                            yield (TreeDiffSide::Left, key, node);
                        }
                    }
                    (TreeNode::Leaf(left_leaf_node), TreeNode::Branch(right_branch_id)) => {
                        yield (TreeDiffSide::Left, key.clone(), Ok(left_leaf_node.clone()));

                        let leaves = tree_leaves(provider, right_branch_id, key);

                        tokio::pin!(leaves);

                        while let Some((key, node)) = leaves.next().await {
                            yield (TreeDiffSide::Right, key, node);
                        }
                    }
                }
            } else if left_item.0 < right_item.0 {
                left_children = &left_children[1..];
                let key = base_key.join(&left_item.0);

                match &left_item.1 {
                    TreeNode::Branch(left_branch_id) => {
                        // If the left is not a prefix of the right node or if
                        // the right node is not a branch, then we can conclude
                        // the whole left is different and we return all its
                        // leaves.
                        match &right_item.1 {
                            TreeNode::Branch(right_branch_id) => {
                                // If the right branch has the left one as a
                                // prefix, we push the candidate pair to the
                                // stack for further processing.
                                if right_item.0.has_prefix(&left_item.0) {
                                    // If we push the right branch to the
                                    // stack, we need to stop processing it
                                    // right away.
                                    right_children = &right_children[1..];

                                    stack.push_back((
                                        (key, left_branch_id.clone()),
                                        (base_key.join(&right_item.0), right_branch_id.clone()),
                                    ));
                                } else {
                                    let leaves = tree_leaves(provider, left_branch_id, key);

                                    tokio::pin!(leaves);

                                    while let Some((key, node)) = leaves.next().await {
                                        yield (TreeDiffSide::Left, key, node);
                                    }
                                }
                            }
                            TreeNode::Leaf(_) => {
                                let leaves = tree_leaves(provider, left_branch_id, key);

                                tokio::pin!(leaves);

                                while let Some((key, node)) = leaves.next().await {
                                    yield (TreeDiffSide::Left, key, node);
                                }
                            }
                        }
                    }
                    TreeNode::Leaf(left_leaf_node) => {
                        yield (TreeDiffSide::Left, key, Ok(left_leaf_node.clone()));
                    }
                }
            } else {
                right_children = &right_children[1..];
                let key = base_key.join(&right_item.0);

                match &right_item.1 {
                    TreeNode::Branch(right_branch_id) => {
                        // If the right is not a prefix of the left node or if
                        // the left node is not a branch, then we can conclude
                        // the whole right is different and we return all its
                        // leaves.
                        match &left_item.1 {
                            TreeNode::Branch(left_branch_id) => {
                                // If the left branch has the right one as a
                                // prefix, we push the candidate pair to the
                                // stack for further processing.
                                if left_item.0.has_prefix(&right_item.0) {
                                    // If we push the left branch to the
                                    // stack, we need to stop processing it
                                    // right away.
                                    left_children = &left_children[1..];

                                    stack.push_back((
                                        (base_key.join(&left_item.0), left_branch_id.clone()),
                                        (key, right_branch_id.clone()),
                                    ));
                                } else {
                                    let leaves = tree_leaves(provider, right_branch_id, key);

                                    tokio::pin!(leaves);

                                    while let Some((key, node)) = leaves.next().await {
                                        yield (TreeDiffSide::Right, key, node);
                                    }
                                }
                            }
                            TreeNode::Leaf(_) => {
                                let leaves = tree_leaves(provider, right_branch_id, key);

                                tokio::pin!(leaves);

                                while let Some((key, node)) = leaves.next().await {
                                    yield (TreeDiffSide::Right, key, node);
                                }
                            }
                        }
                    }
                    TreeNode::Leaf(right_leaf_node) => {
                        yield (TreeDiffSide::Right, key, Ok(right_leaf_node.clone()));
                    }
                }
            }
        }

        // Let's process the leftover items of both nodes.
        //
        // Note: Only one of those for loops should ever run at a given time.

        for (local_key, node) in left_children {
            let key = base_key.join(&local_key);

            match &node {
                TreeNode::Branch(left_branch_id) => {
                    let leaves = tree_leaves(provider, left_branch_id, key);

                    tokio::pin!(leaves);

                    while let Some((key, node)) = leaves.next().await {
                        yield (TreeDiffSide::Left, key, node);
                    }
                }
                TreeNode::Leaf(left_leaf_node) => {
                    yield (TreeDiffSide::Left, key, Ok(left_leaf_node.clone()));
                }
            }
        }

        for (local_key, node) in right_children {
            let key = base_key.join(&local_key);

            match &node {
                TreeNode::Branch(right_branch_id) => {
                    let leaves = tree_leaves(provider, right_branch_id, key);

                    tokio::pin!(leaves);

                    while let Some((key, node)) = leaves.next().await {
                        yield (TreeDiffSide::Right, key, node);
                    }
                }
                TreeNode::Leaf(right_leaf_node) => {
                    yield (TreeDiffSide::Right, key, Ok(right_leaf_node.clone()));
                }
            }
        }
    }
}

type DiffStack = VecDeque<((IndexKey, TreeIdentifier), (IndexKey, TreeIdentifier))>;

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

/// Represent a branch information.
#[derive(Debug)]
pub struct TreeBranchInfo<'a> {
    pub parent_id: &'a TreeIdentifier,
    pub key: &'a IndexKey,
    pub local_key: &'a IndexKey,
    pub branch_id: &'a TreeIdentifier,
    pub branch: &'a Tree,
}

/// Represent a branch information.
#[derive(Debug)]
pub struct TreeLeafInfo<'a> {
    pub parent_id: &'a TreeIdentifier,
    pub key: &'a IndexKey,
    pub local_key: &'a IndexKey,
    pub leaf_node: &'a TreeLeafNode,
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

    async fn visit_branch(&mut self, _info: TreeBranchInfo<'_>) -> Result<TreeVisitorAction> {
        Ok(TreeVisitorAction::Continue)
    }

    async fn visit_leaf(&mut self, _info: TreeLeafInfo<'_>) -> Result<()> {
        Ok(())
    }

    async fn visit_done(&mut self, _root_id: &TreeIdentifier) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeDiffSide {
    Left,
    Right,
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
            Ok(idx) => Some(std::mem::replace(&mut self.children[idx], (key.into(), node)).1),
            Err(idx) => {
                self.children.insert(idx, (key.into(), node));

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

    /// Returns the children of the tree.
    pub fn children(&self) -> &Vec<(IndexKey, TreeNode)> {
        &self.children
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
    use std::collections::BTreeMap;

    use async_recursion::async_recursion;

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
                "a".into(),
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

    fn leaf(v: &str) -> TreeLeafNode {
        TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(v.as_bytes())))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum MiniTree {
        Branch {
            children: BTreeMap<Vec<u8>, Box<Self>>,
        },
        Leaf {
            data: Vec<u8>,
        },
    }

    impl MiniTree {
        fn append_child(&mut self, key: &[u8], child: Self) {
            match self {
                MiniTree::Branch { children } => {
                    children.insert(key.to_vec(), Box::new(child));
                }
                MiniTree::Leaf { .. } => panic!("Cannot append child to leaf"),
            }
        }

        async fn write(self, provider: &Provider) -> Result<TreeIdentifier> {
            if let TreeNode::Branch(tree_id) = self.write_node(provider).await? {
                Ok(tree_id)
            } else {
                panic!("expected branch");
            }
        }

        #[async_recursion]
        async fn write_node(self, provider: &Provider) -> Result<TreeNode> {
            Ok(match self {
                Self::Branch { children } => {
                    let mut new_children = Vec::with_capacity(children.len());

                    for (key, mini_tree) in children {
                        let node = mini_tree.write_node(provider).await?;
                        new_children.push((key.into(), node));
                    }

                    let tree = Tree {
                        count: new_children.len(),
                        total_size: 0, // We don't care here.
                        children: new_children,
                    };
                    let tree_id = provider.write_tree(&tree).await?;

                    TreeNode::Branch(tree_id)
                }
                Self::Leaf { data } => {
                    let id = Identifier::new_data(&data);
                    let res_id = ResourceIdentifier(id);
                    let tree_leaf_node = TreeLeafNode::Resource(res_id);

                    TreeNode::Leaf(tree_leaf_node)
                }
            })
        }
    }

    async fn parse_tree(provider: &Provider, s: &str) -> Result<TreeIdentifier> {
        let root = MiniTree::Branch {
            children: BTreeMap::new(),
        };
        let mut stack = vec![(vec![], root)];
        let mut offset = None;

        for line in s.split('\n') {
            let line = line.trim_end();

            let (depth, key, value) = match line.find(|c| c != ' ') {
                Some(mut pos) => {
                    let mut iter = line[pos..].split(':');
                    let hexkey = iter.next().expect("missing key");
                    let key = hex::decode(hexkey).expect("invalid key");
                    let value = iter.next().map(str::trim);

                    if let Some(offset) = offset {
                        pos -= offset;
                    } else {
                        offset = Some(pos);
                        pos = 0;
                    }

                    (pos / 2, key, value)
                }
                None => continue,
            };

            while stack.len() > depth + 1 {
                let (key, node) = stack.pop().unwrap();
                stack.last_mut().unwrap().1.append_child(&key, node);
            }

            match value {
                Some(value) => {
                    stack.last_mut().unwrap().1.append_child(
                        &key,
                        MiniTree::Leaf {
                            data: value.as_bytes().to_vec(),
                        },
                    );
                }
                None => {
                    stack.push((
                        key,
                        MiniTree::Branch {
                            children: BTreeMap::new(),
                        },
                    ));
                }
            }
        }

        while stack.len() > 1 {
            let (key, node) = stack.pop().unwrap();
            stack.last_mut().unwrap().1.append_child(&key, node);
        }

        stack.pop().expect("missing root").1.write(provider).await
    }

    #[tokio::test]
    async fn test_tree_diff_identical_leaves_with_different_structure() {
        let provider = Provider::new_in_memory();

        // Let's craft some very specific trees.
        let left_tree_id = parse_tree(
            &provider,
            r#"
            00
              0000
                  00: a
            01
              0000
                  00: b
                  01: c
            "#,
        )
        .await
        .unwrap();

        let right_tree_id = parse_tree(
            &provider,
            r#"
            0000
                0000: a
            0100
                00
                  00: b
                  01: c
            "#,
        )
        .await
        .unwrap();

        let base_key = IndexKey::default();
        let mut leaves = tree_diff(&provider, &base_key, &left_tree_id, &right_tree_id)
            .map(|(side, index_key, leaf)| leaf.map(|leaf| (side, index_key, leaf)))
            .collect::<Result<Vec<_>>>()
            .await
            .unwrap();

        leaves.sort();

        assert_eq!(leaves, vec![],);
    }

    #[tokio::test]
    async fn test_tree_diff_with_differences() {
        let provider = Provider::new_in_memory();

        // Let's craft some very specific trees.
        let left_tree_id = parse_tree(
            &provider,
            r#"
            00
              0000
                  00: a
                  01: x
            01
              0000
                  00: b
                  01: c
            02
              000000: y
            "#,
        )
        .await
        .unwrap();

        let right_tree_id = parse_tree(
            &provider,
            r#"
            0000
                0000: a
            0100
                00
                  00: z
                  01: c
            "#,
        )
        .await
        .unwrap();

        let base_key = IndexKey::default();
        let mut leaves = tree_diff(&provider, &base_key, &left_tree_id, &right_tree_id)
            .map(|(side, index_key, leaf)| leaf.map(|leaf| (side, index_key, leaf)))
            .collect::<Result<Vec<_>>>()
            .await
            .unwrap();

        leaves.sort();

        use TreeDiffSide::{Left, Right};

        assert_eq!(
            leaves,
            vec![
                (Left, vec![0, 0, 0, 1].into(), leaf("x")),
                (Left, vec![1, 0, 0, 0].into(), leaf("b")),
                (Left, vec![2, 0, 0, 0].into(), leaf("y")),
                (Right, vec![1, 0, 0, 0].into(), leaf("z")),
            ]
        );
    }

    #[tokio::test]
    async fn test_tree_diff_same_tree_different_values() {
        let provider = Provider::new_in_memory();

        // Let's craft some very specific trees.
        let left_tree_id = parse_tree(
            &provider,
            r#"
            00
              0000
                  00: a
            01
              0000
                  00: b
                  01: c
            "#,
        )
        .await
        .unwrap();

        let right_tree_id = parse_tree(
            &provider,
            r#"
            00
              0000
                  00: a'
            01
              0000
                  00: b'
                  01: c'
            "#,
        )
        .await
        .unwrap();

        let base_key = IndexKey::default();
        let mut leaves = tree_diff(&provider, &base_key, &left_tree_id, &right_tree_id)
            .map(|(side, index_key, leaf)| leaf.map(|leaf| (side, index_key, leaf)))
            .collect::<Result<Vec<_>>>()
            .await
            .unwrap();

        leaves.sort();

        use TreeDiffSide::{Left, Right};

        assert_eq!(
            leaves,
            vec![
                (Left, vec![0, 0, 0, 0].into(), leaf("a")),
                (Left, vec![1, 0, 0, 0].into(), leaf("b")),
                (Left, vec![1, 0, 0, 1].into(), leaf("c")),
                (Right, vec![0, 0, 0, 0].into(), leaf("a'")),
                (Right, vec![1, 0, 0, 0].into(), leaf("b'")),
                (Right, vec![1, 0, 0, 1].into(), leaf("c'")),
            ]
        );
    }
}
