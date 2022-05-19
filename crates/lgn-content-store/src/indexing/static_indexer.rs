use std::{
    collections::{BTreeMap, VecDeque},
    ops::{Bound, RangeBounds},
    pin::Pin,
};

use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::{indexing::TreeWriter, Provider};

use super::{
    tree::{TreeIdentifier, TreeLeafNode},
    BasicIndexer, Error, IndexKey, IndexKeyBound, IndexPath, IndexPathItem, OrderedIndexer, Result,
    SearchResult, Tree, TreeNode, TreeReader,
};

/// A `StaticIndexer` is an indexer that adds resources according to the prefix
/// of the index key.
///
/// The index keys are required to have a fixed length known in advance.
///
/// This indexer allows only for exact searches and full-listing of its leaves.
///
/// # Usage
///
/// This kind of indexer is perfect to store a list of resources according to a
/// fixed-sized identifier (OID for instance).
///
/// It can be used either as a direct OID to resource index, or as a reverse
/// dependency index. In the latter case, its leaves likely point to the root of
/// a sub-index tree (possibly another `StaticIndexer`).
///
/// # Speed and algorithmic complexity
///
/// This tree is designed such that the children of each layer are guaranteed to
/// have keys of the exact same size. The children of two different layers may
/// very well have different key sizes though, even if the layers are at the
/// same depth.
///
/// When the tree contains very few values, the tree's depth will be small to
/// allow for faster lookups. As the number of values increases, the tree might
/// rebalance itself by splitting layers with bigger keys into multiple layers
/// with smaller keys. Doing so only affects the rebalanced layers and does not
/// cause all layers to be rewritten. It is still however a rather expensive
/// operation.
///
/// The indexer can be configured to generate smaller layers from the very
/// beginning, at the cost of more roundtrips to the content-store for each
/// lookup. This is especially recommended when the tree is expected to contains
/// a great number of leaves.
///
/// See the notes about `min_children_per_layer` and `max_children_per_layer`
/// for details.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct StaticIndexer {
    /// The expected length of the index keys.
    ///
    /// You won't be able to change it afterwards. Don't try it. Really.
    index_key_length: usize,

    /// The minimum number of children per tree layer.
    ///
    /// This is not a hard constraint but guides the rebalancing algorithm.
    ///
    /// If after a rebalancing operation the number of children per tree layer
    /// is below this value, the rebalancing will be performed again with a
    /// bigger index key length to increase the number of children per tree
    /// layer.
    ///
    /// A good value for this and the default is 2, which will avoid single-node
    /// trees as much as possible.
    ///
    /// Setting this to 1 will prevent the rebalancing algorithm from merging
    /// layers, which is almost never desirable as it causes excessive
    /// roundtrips to the content-store for all operations. This is only ever
    /// useful for very large trees. In such a case, you also probably need to
    /// set `max_children_per_layer` to 1 as well, which will garantee that all
    /// layers have a key length of 1.
    #[serde(default = "StaticIndexer::default_min_children_per_layer")]
    min_children_per_layer: usize,

    /// The maximum number of children per tree layer.
    ///
    /// This is just a hint. If the value is below 256, balancing the tree may
    /// yield more children than `max_children_per_layer` in some cases.
    ///
    /// Using a value above 4096 can lead to a very large index tree nodes,
    /// which can cause higher memory footprints and contribute to cache misses
    /// and high indexing latency. In particular, using a large value may cause
    /// layer merging operations to be extremely slow, which happens when
    /// elements are removed from the tree. Use with caution.
    ///
    /// Setting a value of 1 for both this and `min_children_per_layer` is
    /// recommended for trees that are expected to have many resources to avoid
    /// rebalancing.
    #[serde(default = "StaticIndexer::default_max_children_per_layer")]
    max_children_per_layer: usize,
}

impl StaticIndexer {
    /// Create a new static indexer with the specified key size.
    ///
    /// By default, the indexer will use a minimum of 2 children and a maximum
    /// of 256 children per layer.
    pub fn new(index_key_length: usize) -> Self {
        Self {
            index_key_length,
            min_children_per_layer: Self::default_min_children_per_layer(),
            max_children_per_layer: Self::default_max_children_per_layer(),
        }
    }

    /// Set the layer constraints.
    ///
    /// See `min_children_per_layer` and `max_children_per_layer` for details.
    ///
    /// # Panics
    ///
    /// Panics if `min_children_per_layer` is bigger than `max_children_per_layer`.
    /// Panics if `min_children_per_layer` smaller than 1.
    /// Panics if `max_children_per_layer` smaller than 2.
    pub fn set_layer_constraints(
        &mut self,
        min_children_per_layer: usize,
        max_children_per_layer: usize,
    ) {
        assert!(
            min_children_per_layer >= 1,
            "min_children_per_layer must be at least 1"
        );
        assert!(
            max_children_per_layer >= 2,
            "max_children_per_layer must be at least 2"
        );

        assert!(
            min_children_per_layer <= max_children_per_layer,
            "min_children_per_layer ({}) must be <= max_children_per_layer ({})",
            min_children_per_layer,
            max_children_per_layer
        );

        self.min_children_per_layer = min_children_per_layer;
        self.max_children_per_layer = max_children_per_layer;
    }

    fn default_min_children_per_layer() -> usize {
        2
    }

    fn default_max_children_per_layer() -> usize {
        256
    }

    fn check_index_key_for_leaf(&self, index_key: &IndexKey) -> Result<()> {
        if index_key.len() != self.index_key_length {
            Err(Error::InvalidIndexKey(format!(
                "expected index key length of {} but got `{:?}` ({} byte(s) long)",
                self.index_key_length,
                index_key,
                index_key.len()
            )))
        } else {
            Ok(())
        }
    }

    fn get_local_key_length(tree: &Tree) -> Option<usize> {
        tree.children.first().map(|(k, _)| k.len())
    }

    async fn search<'i>(
        &'i self,
        provider: &'i Provider,
        root: &Tree,
        index_key: &'i IndexKey,
    ) -> Result<SearchResult<'i>> {
        Ok({
            let mut current_node = root.clone();
            let mut remaining_key = index_key.as_slice();
            let mut local_key: &[u8];

            let mut stack = IndexPath::default();

            loop {
                stack.push(IndexPathItem {
                    tree: current_node.clone(),
                    key: remaining_key,
                });

                if let Some(local_key_length) = Self::get_local_key_length(&current_node) {
                    if remaining_key.len() < local_key_length {
                        // We can't possibly find a match as the remaining key is
                        // smaller than any of the children's keys: the search stops
                        // here.
                        //
                        // At this point there might be multiple children prefix
                        // matches for the remaining key, but the API does not allow
                        // us to return multiple results so this must be resolved
                        // later or the tree must be rebalanced.
                        break SearchResult::NotFound(stack);
                    }

                    (local_key, remaining_key) = remaining_key.split_at(local_key_length);

                    match current_node.into_children(local_key) {
                        None => break SearchResult::NotFound(stack),
                        Some(node) => {
                            // We found a children with the local key: let's
                            // replace the key of the last element in the
                            // stack to reflect that.
                            stack.last_mut().unwrap().key = local_key;

                            match node {
                                TreeNode::Leaf(leaf) => {
                                    if remaining_key.is_empty() {
                                        break SearchResult::Leaf(stack, leaf);
                                    }

                                    return Err(Error::CorruptedTree(format!(
                                        "search in the index stopped too early: a leaf node was found at `{}` but a branch was expected",
                                        hex::encode(&index_key[..remaining_key.len()]),
                                    )));
                                }
                                TreeNode::Branch(branch) => {
                                    if remaining_key.is_empty() {
                                        break SearchResult::Branch(stack, branch);
                                    }

                                    current_node = provider.read_tree(&branch).await?;
                                }
                            }
                        }
                    }
                } else {
                    // We don't have any children so we have no idea how
                    // long the keys are at this level since we haven't
                    // decided yet.
                    //
                    // This decision will be made later, when the first
                    // child is added.
                    //
                    // For now, it means we can exit early, as we can't
                    // possibly find what we are looking for.
                    break SearchResult::NotFound(stack);
                }
            }
        })
    }

    async fn split_tree(&self, provider: &Provider, mut tree: Tree) -> Result<Tree> {
        if tree.direct_count() > self.max_children_per_layer {
            // We are sure to have at least one child or we wouldn't have reached
            // this point.
            match Self::get_local_key_length(&tree).unwrap() {
                0 => return Err(Error::CorruptedTree("local key length is zero".to_string())),
                local_key_length => {
                    let mut split_index = 1;

                    let buckets: BTreeMap<IndexKey, Vec<(IndexKey, TreeNode)>> = loop {
                        if split_index >= local_key_length {
                            // Splitting further would yield the current tree.
                            return Ok(tree);
                        }

                        let mut buckets = BTreeMap::new();

                        for (key, item) in &tree.children {
                            let (bucket_key, key) = key.split_at(split_index);

                            (*buckets.entry(bucket_key.into()).or_insert(Vec::new()))
                                .push((key.into(), item.clone()));
                        }

                        // Check if we have enough buckets.
                        if buckets.len() >= self.min_children_per_layer {
                            break buckets;
                        }

                        split_index += 1;
                    };

                    if buckets.len() == tree.direct_count() {
                        // If rebalancing yields the same number of children, we
                        // don't bother.
                        return Ok(tree);
                    }

                    tree.children = Vec::with_capacity(buckets.len());

                    for (key, children) in buckets {
                        let mut bucket_tree = Tree {
                            count: 0,
                            total_size: 0,
                            children,
                        };

                        for (_, child) in &bucket_tree.children {
                            match child {
                                TreeNode::Leaf(leaf) => {
                                    bucket_tree.count += 1;
                                    bucket_tree.total_size +=
                                        provider.read_size(leaf.as_identifier()).await?;
                                }
                                TreeNode::Branch(tree_id) => {
                                    let tree = provider.read_tree(tree_id).await?;
                                    bucket_tree.count += tree.count;
                                    bucket_tree.total_size += tree.total_size;
                                }
                            }
                        }

                        tree.children.push((
                            key,
                            TreeNode::Branch(provider.write_tree(&bucket_tree).await?),
                        ));
                    }
                }
            }
        }

        Ok(tree)
    }

    async fn merge_tree(&self, provider: &Provider, mut tree: Tree) -> Result<Tree> {
        if tree.direct_count() < self.min_children_per_layer
            && tree.count >= self.min_children_per_layer
        {
            let mut children = BTreeMap::new();

            for (key, item) in &tree.children {
                match item {
                    TreeNode::Leaf(_) => {
                        // We can't merge a node with leaves.
                        return Ok(tree);
                    }
                    TreeNode::Branch(tree_id) => {
                        let sub_tree = provider.read_tree(tree_id).await?;

                        for (sub_key, leaf) in sub_tree.children {
                            let local_key = key.join(sub_key);
                            children.insert(local_key, leaf);
                        }

                        // The branch is actually removed from the tree as this point.
                        provider.unwrite(tree_id.as_identifier()).await;
                    }
                }
            }

            tree.children = children.into_iter().collect();

            // We could very well end-up with a tree that has too many elements
            // and needs a split-up.
            tree = self.split_tree(provider, tree).await?;
        }

        Ok(tree)
    }
}

#[async_trait]
impl BasicIndexer for StaticIndexer {
    async fn get_leaf(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &IndexKey,
    ) -> Result<Option<TreeLeafNode>> {
        self.check_index_key_for_leaf(index_key)?;

        let root = provider.read_tree(root_id).await?;

        match self.search(provider, &root, index_key).await? {
            SearchResult::Leaf(_, leaf) => Ok(Some(leaf)),
            SearchResult::Branch(..) | SearchResult::NotFound(..) => Ok(None),
        }
    }

    async fn add_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
        leaf_node: TreeLeafNode,
    ) -> Result<TreeIdentifier> {
        self.check_index_key_for_leaf(index_key)?;

        let root = provider.read_tree(root_id).await?;

        match self.search(provider, &root, index_key).await? {
            SearchResult::Leaf(_, existing_leaf_node) => Err(
                Error::IndexTreeLeafNodeAlreadyExists(index_key.clone(), existing_leaf_node),
            ),
            SearchResult::Branch(..) => Err(Error::CorruptedTree(format!(
                "a branch node with the same key already exists: `{:?}`",
                index_key
            ))),
            SearchResult::NotFound(mut stack) => {
                let size_delta = provider.read_size(leaf_node.as_identifier()).await?;

                // This should always be true since `NotFound` is only returned
                // with a non-empty stack.
                let mut item = stack.pop().expect("stack is not empty");

                let mut node = match Self::get_local_key_length(&item.tree) {
                    None => {
                        // This is the node first child: we can use the whole
                        // remaining key as the child key.
                        TreeNode::Leaf(leaf_node)
                    }
                    Some(local_key_length) => {
                        if local_key_length > item.key.len() {
                            return Err(Error::CorruptedTree(format!(
                                "local key length is larger than the remaining key: {} > {}",
                                local_key_length,
                                item.key.len()
                            )));
                        }

                        if local_key_length == item.key.len() {
                            // The item already has children but our local key has the appropriate size already: just create a single child.
                            TreeNode::Leaf(leaf_node)
                        } else {
                            // The item key is too long for the current local
                            // key: we need to split it and create an
                            // intermediate node.

                            let remaining_key;
                            (item.key, remaining_key) = item.key.split_at(local_key_length);

                            let tree = Tree {
                                count: 1,
                                total_size: size_delta,
                                children: vec![(remaining_key.into(), TreeNode::Leaf(leaf_node))],
                            };

                            let tree_id = provider.write_tree(&tree).await?;

                            TreeNode::Branch(tree_id)
                        }
                    }
                };

                loop {
                    item.tree.count += 1;

                    if let Some(old_node) = item.tree.insert_children(item.key, node) {
                        provider.unwrite(old_node.as_identifier()).await;
                    }

                    item.tree.total_size += size_delta;
                    item.tree = self.split_tree(provider, item.tree).await?;

                    node = TreeNode::Branch(provider.write_tree(&item.tree).await?);

                    if let Some(next) = stack.pop() {
                        item = next;
                    } else {
                        provider.unwrite(root_id.as_identifier()).await;

                        break Ok(node.into_branch().unwrap());
                    }
                }
            }
        }
    }

    async fn replace_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
        leaf_node: TreeLeafNode,
    ) -> Result<(TreeIdentifier, TreeLeafNode)> {
        self.check_index_key_for_leaf(index_key)?;

        let root = provider.read_tree(root_id).await?;

        match self.search(provider, &root, index_key).await? {
            SearchResult::Leaf(mut stack, existing_leaf_node) => {
                if existing_leaf_node == leaf_node {
                    Ok((root_id.clone(), existing_leaf_node))
                } else {
                    let data_size = provider.read_size(leaf_node.as_identifier()).await?;
                    let mut item = stack.pop().expect("stack is not empty");
                    let mut node = TreeNode::Leaf(leaf_node);

                    loop {
                        if let Some(old_node) = item.tree.insert_children(item.key, node) {
                            provider.unwrite(old_node.as_identifier()).await;
                        }

                        item.tree.total_size += data_size;
                        item.tree.total_size -= provider
                            .read_size(existing_leaf_node.as_identifier())
                            .await?;

                        node = TreeNode::Branch(provider.write_tree(&item.tree).await?);

                        if let Some(next) = stack.pop() {
                            item = next;
                        } else {
                            provider.unwrite(root_id.as_identifier()).await;

                            break Ok((node.into_branch().unwrap(), existing_leaf_node));
                        }
                    }
                }
            }
            SearchResult::Branch(..) => Err(Error::CorruptedTree(format!(
                "a branch node was found at `{:?}` which can't be replaced",
                index_key
            ))),
            SearchResult::NotFound(_) => Err(Error::IndexTreeLeafNodeNotFound(index_key.clone())),
        }
    }

    async fn remove_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
    ) -> Result<(TreeIdentifier, TreeLeafNode)> {
        self.check_index_key_for_leaf(index_key)?;

        let root = provider.read_tree(root_id).await?;

        match self.search(provider, &root, index_key).await? {
            SearchResult::Leaf(mut stack, existing_leaf_node) => {
                let mut item = stack.pop().expect("stack is not empty");

                loop {
                    if let Some(old_node) = item.tree.remove_children(item.key) {
                        provider.unwrite(old_node.as_identifier()).await;
                    }

                    if !item.tree.is_empty() {
                        break;
                    }

                    if let Some(next) = stack.pop() {
                        item = next;
                    } else {
                        // If we get here, it means we reached the root of the
                        // stack with an empty tree, and we have no choice but
                        // to return.
                        //
                        // This should always return an empty tree.
                        provider.unwrite(root_id.as_identifier()).await;

                        return Ok((
                            provider.write_tree(&Tree::default()).await?,
                            existing_leaf_node,
                        ));
                    }
                }

                loop {
                    item.tree.count -= 1;
                    item.tree.total_size -= provider
                        .read_size(existing_leaf_node.as_identifier())
                        .await?;

                    item.tree = self.merge_tree(provider, item.tree).await?;

                    let node = TreeNode::Branch(provider.write_tree(&item.tree).await?);

                    if let Some(next) = stack.pop() {
                        item = next;
                    } else {
                        provider.unwrite(root_id.as_identifier()).await;

                        break Ok((node.into_branch().unwrap(), existing_leaf_node));
                    }

                    if let Some(old_node) = item.tree.insert_children(item.key, node) {
                        provider.unwrite(old_node.as_identifier()).await;
                    }
                }
            }
            SearchResult::Branch(..) => Err(Error::CorruptedTree(format!(
                "a branch node was found at `{:?}` which can't be removed",
                index_key
            ))),
            SearchResult::NotFound(_) => Err(Error::IndexTreeLeafNodeNotFound(index_key.clone())),
        }
    }
}

#[async_trait]
impl OrderedIndexer for StaticIndexer {
    async fn enumerate_leaves_in_range<'s, T, R>(
        &'s self,
        provider: &'s Provider,
        root_id: &'s TreeIdentifier,
        range: R,
    ) -> Result<Pin<Box<dyn Stream<Item = (IndexKey, Result<TreeLeafNode>)> + Send + 's>>>
    where
        T: Into<IndexKey> + Clone,
        R: RangeBounds<T> + Send + 's,
    {
        let start_bound = range.start_bound().as_index_key_bound();
        let end_bound = range.end_bound().as_index_key_bound();

        match &start_bound {
            Bound::Included(index_key) | Bound::Excluded(index_key) => {
                self.check_index_key_for_leaf(index_key)?;
            }
            Bound::Unbounded => {}
        };
        match &end_bound {
            Bound::Included(index_key) | Bound::Excluded(index_key) => {
                self.check_index_key_for_leaf(index_key)?;
            }
            Bound::Unbounded => {}
        };

        let mut trees = VecDeque::new();

        Ok(Box::pin(stream! {
            let root = provider.read_tree(root_id).await.unwrap();
            trees.push_back((IndexKey::default(), root));

            while let Some((prefix, current_tree)) = trees.pop_front() {
                for (key, node) in current_tree.children {
                    let new_prefix = prefix.join(key);

                    // Until we can perform a complete comparison, we need to be
                    // conservative.
                    if new_prefix.len() < self.index_key_length {
                        match &start_bound {
                            Bound::Included(index_key) | Bound::Excluded(index_key) => {
                                // New prefix is too small: we continue.
                                if new_prefix.as_slice() < &index_key[..new_prefix.len()] {
                                    continue;
                                }
                            },
                            Bound::Unbounded => {}
                        };
                        match &end_bound {
                            Bound::Included(index_key) | Bound::Excluded(index_key) => {
                                // New prefix is too small: we continue.
                                if new_prefix.as_slice() > &index_key[..new_prefix.len()] {
                                    // We are iterating over the keys in order,
                                    // so the next values in the current node
                                    // will be too large too.
                                    break;
                                }
                            },
                            Bound::Unbounded => {}
                        };
                    } else {
                        match &start_bound {
                            Bound::Included(index_key) => {
                                // New prefix is too small: we continue.
                                if &new_prefix < index_key {
                                    continue;
                                }
                            },
                            Bound::Excluded(index_key) => {
                                // New prefix is too small or equal: we continue.
                                if &new_prefix <= index_key {
                                    continue;
                                }
                            },
                            Bound::Unbounded => {}
                        };
                        match &end_bound {
                            Bound::Included(index_key) => {
                                // New prefix is too small: we continue.
                                if &new_prefix > index_key {
                                    // We are iterating over the keys in order,
                                    // so the next values in the current node
                                    // will be too large too.
                                    break;
                                }
                            },
                            Bound::Excluded(index_key) => {
                                // New prefix is too small or equal: we continue.
                                if &new_prefix >= index_key {
                                    // We are iterating over the keys in order,
                                    // so the next values in the current node
                                    // will be too large too.
                                    break;
                                }
                            },
                            Bound::Unbounded => {}
                        };
                    };

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
        }))
    }
}
#[cfg(test)]
mod tests {
    use crate::{indexing::ResourceIdentifier, Identifier};

    use tokio_stream::StreamExt;

    use super::*;

    macro_rules! read_tree {
        ($provider:expr, $tree_id:expr) => {{
            $provider.read_tree(&$tree_id).await.unwrap()
        }};
    }

    macro_rules! get_leaf {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr) => {{
            $indexer
                .get_leaf(&$provider, &$tree_id, &IndexKey::from_hex(&$key).unwrap())
                .await
                .unwrap()
        }};
    }

    macro_rules! assert_leaf_does_not_exist {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr) => {{
            assert!(get_leaf!($indexer, $provider, $tree_id, $key).is_none());
        }};
    }

    macro_rules! assert_leaf {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr, $leaf:expr) => {{
            assert_eq!(
                Some(&$leaf),
                get_leaf!($indexer, $provider, $tree_id, $key).as_ref()
            );
        }};
    }

    macro_rules! resource_leaf {
        ($content:expr) => {{
            #[allow(clippy::string_lit_as_bytes)]
            TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(
                $content.as_bytes(),
            )))
        }};
    }

    macro_rules! add_leaf_hex {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr, $content:expr) => {{
            let leaf = resource_leaf!($content);

            (
                $indexer
                    .add_leaf(
                        &$provider,
                        &$tree_id,
                        &IndexKey::from_hex(&$key).unwrap(),
                        leaf.clone(),
                    )
                    .await
                    .unwrap(),
                leaf,
            )
        }};
    }

    macro_rules! add_leaf {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr, $content:expr) => {{
            let leaf = resource_leaf!($content);

            (
                $indexer
                    .add_leaf(&$provider, &$tree_id, &($key.into()), leaf.clone())
                    .await
                    .unwrap(),
                leaf,
            )
        }};
    }

    macro_rules! assert_add_leaf_already_exists {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr, $content:expr) => {{
            let leaf = resource_leaf!($content);
            let key = IndexKey::from_hex(&$key).unwrap();

            match $indexer
                .add_leaf(&$provider, &$tree_id, &key, leaf.clone())
                .await
                .unwrap_err()
            {
                Error::IndexTreeLeafNodeAlreadyExists(k, l) => {
                    assert_eq!(k, key);
                    assert_eq!(l, leaf);
                }
                err => panic!("unexpected error: {:#?}", err),
            }
        }};
    }

    macro_rules! assert_replace_leaf {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr, $old_content:expr, $content:expr) => {{
            let leaf = resource_leaf!($content);

            let (tree_id, old_leaf) = $indexer
                .replace_leaf(
                    &$provider,
                    &$tree_id,
                    &IndexKey::from_hex(&$key).unwrap(),
                    leaf.clone(),
                )
                .await
                .unwrap();

            assert_eq!(resource_leaf!($old_content), old_leaf);

            (tree_id, leaf)
        }};
    }

    macro_rules! assert_replace_leaf_not_found {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr, $content:expr) => {{
            let leaf = resource_leaf!($content);
            let key = IndexKey::from_hex(&$key).unwrap();

            match $indexer
                .replace_leaf(&$provider, &$tree_id, &key, leaf.clone())
                .await
                .unwrap_err()
            {
                Error::IndexTreeLeafNodeNotFound(k) => {
                    assert_eq!(k, key);
                }
                err => panic!("unexpected error: {:#?}", err),
            };
        }};
    }

    macro_rules! assert_remove_leaf {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr, $old_content:expr) => {{
            let (tree_id, old_leaf) = $indexer
                .remove_leaf(&$provider, &$tree_id, &IndexKey::from_hex(&$key).unwrap())
                .await
                .unwrap();

            assert_eq!(resource_leaf!($old_content), old_leaf);

            tree_id
        }};
    }

    macro_rules! assert_remove_leaf_not_found {
        ($indexer:expr, $provider:expr, $tree_id:expr, $key:expr) => {{
            let key = IndexKey::from_hex(&$key).unwrap();

            match $indexer
                .remove_leaf(&$provider, &$tree_id, &key)
                .await
                .unwrap_err()
            {
                Error::IndexTreeLeafNodeNotFound(k) => {
                    assert_eq!(k, key);
                }
                err => panic!("unexpected error: {:#?}", err),
            };
        }};
    }

    macro_rules! assert_get_leaves_in_range {
        ($indexer:expr, $provider:expr, $tree_id:expr, $range:expr, $expected:expr) => {{
            let leaves = $indexer
                .enumerate_leaves_in_range::<u32, _>(&$provider, &$tree_id, $range)
                .await
                .unwrap()
                .map(|(key, node)| node.map(|node| (key, node)))
                .collect::<Result<Vec<_>>>()
                .await
                .unwrap();

            assert_eq!(&leaves, &$expected);
        }};
    }

    #[tokio::test]
    async fn test_static_indexer() {
        let provider = Provider::new_in_memory();
        let idx = StaticIndexer {
            index_key_length: 4,
            min_children_per_layer: 2,
            max_children_per_layer: 4,
        };

        // This is our starting point: we write an empty tree.
        //
        // In all likelyhood, the generated identifier will benefit from
        // small-content optimization and not actually be written anywhere.
        let tree_id = provider.write_tree(&Tree::default()).await.unwrap();

        // Let's perform a search in the empty tree. It should yield no result.
        assert_leaf_does_not_exist!(idx, provider, tree_id, "00000000");

        // Let's insert a new leaf node.
        let (tree_id, leaf_node1) = add_leaf_hex!(idx, provider, tree_id, "00000000", "a");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 1);
        assert_eq!(tree.total_size(), 1);

        // Let's perform a search in the resulting tree. It should yield the leaf node.
        assert_leaf!(idx, provider, tree_id, "00000000", leaf_node1);

        // Adding the same leaf node with the exact same key should yield a very specific error:
        // error and no changes to the tree.
        assert_add_leaf_already_exists!(idx, provider, tree_id, "00000000", "a");

        // Let's insert a new leaf node.
        let (tree_id, leaf_node2) = add_leaf_hex!(idx, provider, tree_id, "00000001", "bigger");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 2);
        assert_eq!(tree.total_size, 7);

        // Let's perform a search in the resulting tree. It should yield the leaf node.
        assert_leaf!(idx, provider, tree_id, "00000001", leaf_node2);

        // We add two more nodes.
        let (tree_id, _) = add_leaf_hex!(idx, provider, tree_id, "00000002", "node3");
        let (tree_id, _) = add_leaf_hex!(idx, provider, tree_id, "00000003", "node4");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 4);
        assert_eq!(tree.total_size, 17);
        assert_eq!(tree.direct_count(), 4);

        // Adding a fifth node should cause a tree-rebalancing, since we only
        // allow for 4 children per layer in this test.
        //
        // As there are only 2 different prefixes, we expect the tree to have a
        // first layer with 2 children.
        let (tree_id, _) = add_leaf_hex!(idx, provider, tree_id, "00000100", "node5");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 5);
        assert_eq!(tree.total_size, 22);
        assert_eq!(tree.direct_count(), 2); // The root tree should have been rebalanced.

        // The same search than before should still work.
        assert_leaf!(idx, provider, tree_id, "00000001", leaf_node2);

        // We keep adding nodes with different prefixes to trigger a top-level
        // rebalancing.
        let (tree_id, _) = add_leaf_hex!(idx, provider, tree_id, "01000000", "node6");
        let (tree_id, _) = add_leaf_hex!(idx, provider, tree_id, "02000000", "node7");
        let (tree_id, _) = add_leaf_hex!(idx, provider, tree_id, "03000000", "node8");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 8);
        assert_eq!(tree.total_size, 37);
        assert_eq!(tree.direct_count(), 4); // The root tree should have been rebalanced.

        // Perform a replacement.
        let (tree_id, leaf_node2) =
            assert_replace_leaf!(idx, provider, tree_id, "00000001", "bigger", "node2");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 8);
        assert_eq!(tree.total_size, 36); // Notice how the size of the tree has decreased.

        // The same search than before should still work.
        assert_leaf!(idx, provider, tree_id, "00000001", leaf_node2);

        // Perform an illegal replacement.
        assert_replace_leaf_not_found!(idx, provider, tree_id, "00000099", "node99");

        // Remove the node5.
        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "00000100", "node5");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 7);
        assert_eq!(tree.total_size, 31); // Notice how the size of the tree has decreased.

        // Perform an illegal removal.
        assert_remove_leaf_not_found!(idx, provider, tree_id, "00000099");

        // Remove the nodes 6, 7 and 8, which should cause a rebalancing.
        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "01000000", "node6");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 6);
        assert_eq!(tree.total_size, 26); // Notice how the size of the tree has decreased.

        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "02000000", "node7");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 5);
        assert_eq!(tree.total_size, 21); // Notice how the size of the tree has decreased.

        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "03000000", "node8");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 4);
        assert_eq!(tree.total_size, 16); // Notice how the size of the tree has decreased.
        assert_eq!(tree.direct_count(), 4); // We make sure the tree was rebalanced.

        // Remove all remaining nodes to test for the final empty tree.

        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "00000000", "a");
        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "00000001", "node2");
        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "00000002", "node3");
        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "00000003", "node4");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 0);
        assert_eq!(tree.total_size, 0);

        //crate::indexing::GraphvizVisitor::create("tree.dot")
        //    .await
        //    .unwrap()
        //    .visit(&ct, &tree_id)
        //    .await
        //    .unwrap();

        // The only identifier that should be referenced is the root.
        let ids = provider.referenced().await;
        assert_eq!(&ids, &[tree_id.as_identifier().clone()]);
    }

    #[tokio::test]
    async fn test_static_indexer_range_search() {
        let provider = Provider::new_in_memory();
        let idx = StaticIndexer {
            index_key_length: 4,
            min_children_per_layer: 2,
            max_children_per_layer: 4,
        };

        let tree_id = provider.write_tree(&Tree::default()).await.unwrap();
        let (tree_id, _) = add_leaf!(idx, provider, tree_id, 1_u32, "one");
        let (tree_id, _) = add_leaf!(idx, provider, tree_id, 2_u32, "two");
        let (tree_id, _) = add_leaf!(idx, provider, tree_id, 8_u32, "eight");
        let (tree_id, _) = add_leaf!(idx, provider, tree_id, 256_u32, "two hundred and fifty-six");
        let (tree_id, _) = add_leaf!(idx, provider, tree_id, 512_u32, "five hundred and twelve");

        assert_get_leaves_in_range!(idx, provider, tree_id, 0..1, []);
        assert_get_leaves_in_range!(
            idx,
            provider,
            tree_id,
            0..=1,
            [(1.into(), resource_leaf!("one"))]
        );
        assert_get_leaves_in_range!(
            idx,
            provider,
            tree_id,
            ..2,
            [(1.into(), resource_leaf!("one"))]
        );
        assert_get_leaves_in_range!(
            idx,
            provider,
            tree_id,
            2..300,
            [
                (2.into(), resource_leaf!("two")),
                (8.into(), resource_leaf!("eight")),
                (256.into(), resource_leaf!("two hundred and fifty-six")),
            ]
        );
        assert_get_leaves_in_range!(
            idx,
            provider,
            tree_id,
            2..,
            [
                (2.into(), resource_leaf!("two")),
                (8.into(), resource_leaf!("eight")),
                (256.into(), resource_leaf!("two hundred and fifty-six")),
                (512.into(), resource_leaf!("five hundred and twelve")),
            ]
        );
    }
}
