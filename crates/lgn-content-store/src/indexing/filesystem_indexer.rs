use std::collections::VecDeque;

use async_stream::stream;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::{indexing::TreeWriter, Provider};

use super::{
    tree::{TreeIdentifier, TreeLeafNode},
    Error, IndexKey, IndexPath, IndexPathItem, IntoIndexKey, Result, SearchResult, Tree, TreeNode,
    TreeReader,
};

/// A `FilesystemIndexer` is an indexer that adds resources according to a
/// virtual filesystem path.
///
/// The index keys can be of any size but are expected to be UTF-8 encoded
/// strings.
///
/// This indexer allows only for exact searches, full-listing of its leaves as
/// well as directory browsing.
///
/// # Usage
///
/// This kind of indexer is perfect to store a human-organized list of resources
/// and display or browser them in a filesystem-like way.
///
/// # Speed and algorithmic complexity
///
/// This tree is designed without any balancing functionality. Addition, updates
/// and removal are thus reasonably fast, but the responsibility of not
/// overloading branches (directories) is left to the user.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FilesystemIndexer {
    /// The separator used to separate the path elements.
    #[serde(default = "FilesystemIndexer::default_path_separator")]
    path_separator: u8,

    /// Whether to keep empty directories when removing a resource.
    #[serde(default)]
    keep_empty_branches: bool,
}

impl Default for FilesystemIndexer {
    fn default() -> Self {
        Self::new(Self::default_path_separator())
    }
}

impl FilesystemIndexer {
    /// Create a new static indexer with the specified key size.
    ///
    /// By default, the indexer will use a minimum of 2 children and a maximum
    /// of 256 children per layer.
    pub fn new(path_separator: u8) -> Self {
        Self {
            path_separator,
            keep_empty_branches: false,
        }
    }

    fn default_path_separator() -> u8 {
        b'/'
    }

    /// Set whether to remove empty directories when removing a resource.
    pub fn set_keep_empty_branches(&mut self, keep_empty_branches: bool) {
        self.keep_empty_branches = keep_empty_branches;
    }

    fn sanitize_index_key<'k>(&self, index_key: &'k [u8]) -> &'k [u8] {
        // If the index key starts with the separator, discard it silently.
        let index_key = if index_key.first() == Some(&self.path_separator) {
            &index_key[1..]
        } else {
            index_key
        };

        if index_key.last() == Some(&self.path_separator) {
            &index_key[..index_key.len() - 1]
        } else {
            index_key
        }
    }

    /// Split the index key into its first path element and the remaining path.
    ///
    /// The index key should have been sanitized before calling this function.
    /// See `sanitize_index_key`.
    fn split_first_index_key<'k>(&self, index_key: &'k [u8]) -> (&'k [u8], &'k [u8]) {
        if let Some(position) = index_key.iter().position(|&b| b == self.path_separator) {
            let (a, b) = index_key.split_at(position);
            (a, &b[1..])
        } else {
            (index_key, &[])
        }
    }

    /// Split the index key into its last path element and the remaining path.
    ///
    /// The index key should have been sanitized before calling this function.
    /// See `sanitize_index_key`.
    fn split_last_index_key<'k>(&self, index_key: &'k [u8]) -> (&'k [u8], &'k [u8]) {
        if let Some(position) = index_key
            .iter()
            .rev()
            .position(|&b| b == self.path_separator)
        {
            let (a, b) = index_key.split_at(index_key.len() - position - 1);
            (a, &b[1..])
        } else {
            (index_key, &[])
        }
    }

    async fn search<'i>(
        &'i self,
        provider: &'i Provider,
        root: &Tree,
        index_key: &'i IndexKey,
    ) -> Result<SearchResult<'i>> {
        Ok({
            let mut current_node = root.clone();
            let mut remaining_key: &[u8] = self.sanitize_index_key(index_key);
            let mut local_key: &[u8];

            let mut stack = IndexPath::default();

            loop {
                stack.push(IndexPathItem {
                    tree: current_node.clone(),
                    key: remaining_key,
                });

                (local_key, remaining_key) = self.split_first_index_key(remaining_key);

                match current_node.into_children(local_key) {
                    None => return Ok(SearchResult::NotFound(stack)),
                    Some(node) => {
                        // We found a children with the local key: let's
                        // replace the key of the last element in the
                        // stack to reflect that.
                        stack.last_mut().unwrap().key = local_key;

                        match node {
                            TreeNode::Leaf(leaf) => {
                                if !remaining_key.is_empty() {
                                    return Err(Error::CorruptedTree(format!(
                                        "search in the index stopped too early: a leaf node was found at `{}` but a branch was expected",
                                        hex::encode(&index_key[..remaining_key.len()]),
                                    )));
                                }

                                break SearchResult::Leaf(stack, leaf);
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
            }
        })
    }

    /// Get a leaf or branch node from the tree.
    ///
    /// This function will return `None` if the tree does not contain a leaf
    /// with the specified key.
    ///
    /// # Errors
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    pub async fn get(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &IndexKey,
    ) -> Result<Option<TreeNode>> {
        let root = provider.read_tree(root_id).await?;

        Ok(self.search(provider, &root, index_key).await?.into())
    }

    /// Add a non-existing leaf to the tree.
    ///
    /// # Cost
    ///
    /// Adding a leaf is generally fast.
    ///
    /// # Errors
    ///
    /// If the leaf at the specified index key already exists, this function
    /// will return `Error::IndexTreeLeafNodeAlreadyExists`.
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    pub async fn add_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
        leaf_node: TreeLeafNode,
    ) -> Result<TreeIdentifier> {
        let root = provider.read_tree(root_id).await?;

        match self.search(provider, &root, index_key).await? {
            SearchResult::Leaf(_, existing_leaf_node) => Err(
                Error::IndexTreeLeafNodeAlreadyExists(index_key.clone(), existing_leaf_node),
            ),
            SearchResult::Branch(..) => Err(Error::CorruptedTree(format!(
                "a branch node with the same key already exists: `{}`",
                index_key
            ))),
            SearchResult::NotFound(mut stack) => {
                let size_delta = provider.read_size(leaf_node.as_identifier()).await?;

                // This should always be true since `NotFound` is only returned
                // with a non-empty stack.
                let mut item = stack.pop().expect("stack is not empty");
                let mut node = TreeNode::Leaf(leaf_node);
                let mut local_key: &[u8];

                node = loop {
                    (item.key, local_key) = self.split_last_index_key(item.key);

                    if local_key.is_empty() {
                        break node;
                    }

                    let tree = Tree {
                        count: 1,
                        total_size: size_delta,
                        children: vec![(local_key.into_index_key(), node)],
                    };

                    let tree_id = provider.write_tree(&tree).await?;

                    node = TreeNode::Branch(tree_id);
                };

                loop {
                    item.tree.count += 1;

                    if let Some(old_node) = item.tree.insert_children(item.key, node) {
                        provider.unwrite(old_node.as_identifier()).await?;
                    }

                    item.tree.total_size += size_delta;

                    node = TreeNode::Branch(provider.write_tree(&item.tree).await?);

                    if let Some(next) = stack.pop() {
                        item = next;
                    } else {
                        provider.unwrite(root_id.as_identifier()).await?;

                        break Ok(node.into_branch().unwrap());
                    }
                }
            }
        }
    }

    /// Replace an existing leaf in the tree.
    ///
    /// # Returns
    ///
    /// The new tree and the old leaf replaced by the new one.
    ///
    /// # Cost
    ///
    /// Replacing a leaf is generally fast.
    ///
    /// # Errors
    ///
    /// If the leaf at the specified index key does not exist, this function
    /// will return `Error::IndexTreeLeafNodeNotFound`.
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    pub async fn replace_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
        leaf_node: TreeLeafNode,
    ) -> Result<(TreeIdentifier, TreeLeafNode)> {
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
                            provider.unwrite(old_node.as_identifier()).await?;
                        }

                        item.tree.total_size += data_size;
                        item.tree.total_size -= provider
                            .read_size(existing_leaf_node.as_identifier())
                            .await?;

                        node = TreeNode::Branch(provider.write_tree(&item.tree).await?);

                        if let Some(next) = stack.pop() {
                            item = next;
                        } else {
                            provider.unwrite(root_id.as_identifier()).await?;

                            break Ok((node.into_branch().unwrap(), existing_leaf_node));
                        }
                    }
                }
            }
            SearchResult::Branch(..) => Err(Error::CorruptedTree(format!(
                "a branch node was found at `{}` which can't be replaced",
                index_key
            ))),
            SearchResult::NotFound(_) => Err(Error::IndexTreeLeafNodeNotFound(index_key.clone())),
        }
    }

    /// Remove an existing leaf from the tree.
    ///
    /// # Returns
    ///
    /// The new tree and the old removed leaf.
    ///
    /// # Cost
    ///
    /// Removing a leaf is generally fast.
    ///
    /// If the removal of the leaf causes a parent tree to be empty and the
    /// indexer is configured to remove empty branches, the parent itself
    /// will be removed, recursively.
    ///
    /// # Errors
    ///
    /// If the leaf at the specified index key does not exist, this function
    /// will return `Error::IndexTreeLeafNodeNotFound`.
    ///
    /// If the specified index key is invalid or the tree is corrupted, an error
    /// will be returned.
    pub async fn remove_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
    ) -> Result<(TreeIdentifier, TreeLeafNode)> {
        let root = provider.read_tree(root_id).await?;

        match self.search(provider, &root, index_key).await? {
            SearchResult::Leaf(mut stack, existing_leaf_node) => {
                let mut item = stack.pop().expect("stack is not empty");

                loop {
                    if let Some(old_node) = item.tree.remove_children(item.key) {
                        provider.unwrite(old_node.as_identifier()).await?;
                    }

                    if !item.tree.is_empty() || self.keep_empty_branches {
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
                        provider.unwrite(root_id.as_identifier()).await?;

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

                    let node = TreeNode::Branch(provider.write_tree(&item.tree).await?);

                    if let Some(next) = stack.pop() {
                        item = next;
                    } else {
                        provider.unwrite(root_id.as_identifier()).await?;

                        break Ok((node.into_branch().unwrap(), existing_leaf_node));
                    }

                    if let Some(old_node) = item.tree.insert_children(item.key, node) {
                        provider.unwrite(old_node.as_identifier()).await?;
                    }
                }
            }
            SearchResult::Branch(..) => Err(Error::CorruptedTree(format!(
                "a branch node was found at `{}` which can't be removed",
                index_key
            ))),
            SearchResult::NotFound(_) => Err(Error::IndexTreeLeafNodeNotFound(index_key.clone())),
        }
    }

    /// Returns a stream that iterates over all leaves in the specified tree.
    ///
    /// # Warning
    ///
    /// This method will iterate over the entire tree. If used on a real, large
    /// tree it could actually take a very long time to end. Think twice before
    /// using it.
    pub fn all_leaves<'s>(
        provider: &'s Provider,
        root_id: &'s TreeIdentifier,
    ) -> impl Stream<Item = (IndexKey, Result<TreeLeafNode>)> + 's {
        let mut trees = VecDeque::new();

        stream! {
            let root = provider.read_tree(root_id).await.unwrap();
            trees.push_back((IndexKey::default(), root));

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
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{indexing::ResourceIdentifier, Identifier};

    macro_rules! read_tree {
        ($provider:expr, $tree_id:expr) => {{
            $provider.read_tree(&$tree_id).await.unwrap()
        }};
    }

    macro_rules! get {
        ($indexer:expr, $provider:expr, $tree_id:expr, $path:expr) => {{
            $indexer
                .get(&$provider, &$tree_id, &$path.into())
                .await
                .unwrap()
        }};
    }

    macro_rules! assert_node_does_not_exist {
        ($indexer:expr, $provider:expr, $tree_id:expr, $path:expr) => {{
            assert!(get!($indexer, $provider, $tree_id, $path).is_none());
        }};
    }

    macro_rules! assert_branch {
        ($indexer:expr, $provider:expr, $tree_id:expr, $path:expr) => {{
            match get!($indexer, $provider, $tree_id, $path) {
                Some(TreeNode::Branch(branch)) => branch,
                Some(TreeNode::Leaf(_)) => panic!("found leaf at `{}`", $path),
                _ => panic!("no node found at `{}`", $path),
            }
        }};
    }

    macro_rules! assert_leaf {
        ($indexer:expr, $provider:expr, $tree_id:expr, $path:expr, $leaf:expr) => {{
            assert_eq!(
                Some(TreeNode::Leaf($leaf.clone())),
                get!($indexer, $provider, $tree_id, $path)
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

    macro_rules! add_leaf {
        ($indexer:expr, $provider:expr, $tree_id:expr, $path:expr, $content:expr) => {{
            let leaf = resource_leaf!($content);

            (
                $indexer
                    .add_leaf(&$provider, &$tree_id, &$path.into(), leaf.clone())
                    .await
                    .unwrap(),
                leaf,
            )
        }};
    }

    macro_rules! assert_replace_leaf {
        ($indexer:expr, $provider:expr, $tree_id:expr, $path:expr, $old_content:expr, $content:expr) => {{
            let leaf = resource_leaf!($content);

            let (tree_id, old_leaf) = $indexer
                .replace_leaf(&$provider, &$tree_id, &$path.into(), leaf.clone())
                .await
                .unwrap();

            assert_eq!(resource_leaf!($old_content), old_leaf);

            (tree_id, leaf)
        }};
    }

    macro_rules! assert_remove_leaf {
        ($indexer:expr, $provider:expr, $tree_id:expr, $path:expr, $old_content:expr) => {{
            let (tree_id, old_leaf) = $indexer
                .remove_leaf(&$provider, &$tree_id, &$path.into())
                .await
                .unwrap();

            assert_eq!(resource_leaf!($old_content), old_leaf);

            tree_id
        }};
    }

    #[test]
    fn test_filesystem_indexer_sanitize() {
        let idx = FilesystemIndexer::new(0);

        let b = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(idx.sanitize_index_key(&b), &b);

        let b = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
        assert_eq!(idx.sanitize_index_key(&b), &b[1..10]);

        let b = [0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0];
        assert_eq!(idx.sanitize_index_key(&b), &b[1..12]);
    }

    #[test]
    fn test_filesystem_indexer_split_first() {
        let idx = FilesystemIndexer::new(4);

        let b = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(idx.split_first_index_key(&b), (&b[..4], &b[5..]));

        let b = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 4, 1];
        assert_eq!(idx.split_first_index_key(&b), (&b[..4], &b[5..]));

        let idx = FilesystemIndexer::new(0xA);

        let b = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(idx.split_first_index_key(&b), (&b[..], &[] as &[u8]));
    }

    #[test]
    fn test_filesystem_indexer_split_last() {
        let idx = FilesystemIndexer::new(4);

        let b = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(idx.split_last_index_key(&b), (&b[..4], &b[5..]));

        let b = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 4, 1];
        assert_eq!(idx.split_last_index_key(&b), (&b[..10], &b[11..]));

        let idx = FilesystemIndexer::new(0xA);

        let b = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(idx.split_last_index_key(&b), (&b[..], &[] as &[u8]));
    }

    #[tokio::test]
    async fn test_filesystem_indexer() {
        let provider = Provider::new_in_memory();
        let idx = FilesystemIndexer::default();

        // This is our starting point: we write an empty tree.
        //
        // In all likelyhood, the generated identifier will benefit from
        // small-content optimization and not actually be written anywhere.
        let tree_id = provider.write_tree(&Tree::default()).await.unwrap();

        assert_node_does_not_exist!(idx, provider, tree_id, "/fruits/pear.txt");
        assert_node_does_not_exist!(idx, provider, tree_id, "fruits/pear.txt");

        let (tree_id, _) = add_leaf!(idx, provider, tree_id, "/fruits/apple.txt", "apple");
        let (tree_id, pear_node) = add_leaf!(idx, provider, tree_id, "/fruits/pear.txt", "pear");
        let (tree_id, _) = add_leaf!(idx, provider, tree_id, "/fruits/banana.txt", "banana");
        let (tree_id, _) = add_leaf!(idx, provider, tree_id, "/vegetables/tomato.txt", "tomato");

        //let visitor = crate::indexing::GraphvizVisitor::create("tree.dot")
        //    .await
        //    .unwrap();
        //tree_id.visit(&ct, visitor).await.unwrap();

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 4);
        assert_eq!(tree.total_size(), 21);

        // Let's perform a search in the resulting tree. It should yield the leaf node.
        assert_leaf!(idx, provider, tree_id, "/fruits/pear.txt", pear_node);
        assert_leaf!(idx, provider, tree_id, "fruits/pear.txt", pear_node);

        // Let's do the same for a branch.
        assert_branch!(idx, provider, tree_id, "/vegetables/");
        assert_branch!(idx, provider, tree_id, "/vegetables");
        assert_branch!(idx, provider, tree_id, "vegetables/");
        assert_branch!(idx, provider, tree_id, "vegetables");

        // Let's update a leaf node: a tomato is not a vegetable. It's a fruit.
        let (tree_id, _) = assert_replace_leaf!(
            idx,
            provider,
            tree_id,
            "/vegetables/tomato.txt",
            "tomato",
            "ERROR"
        );

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 4);
        assert_eq!(tree.total_size(), 20);

        // Let's remove a leaf node.
        let tree_id =
            assert_remove_leaf!(idx, provider, tree_id, "/vegetables/tomato.txt", "ERROR");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 3);
        assert_eq!(tree.total_size(), 15);

        // Remove all the nodes.
        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "/fruits/apple.txt", "apple");
        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "/fruits/banana.txt", "banana");
        let tree_id = assert_remove_leaf!(idx, provider, tree_id, "/fruits/pear.txt", "pear");

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 0);
        assert_eq!(tree.total_size(), 0);

        // There should be no identifiers left to pop, as we went back to the
        // original tree.
        let ids = provider.pop_referenced_identifiers();

        assert_eq!(&ids, &[]);
    }
}
