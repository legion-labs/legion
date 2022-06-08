use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{indexing::TreeWriter, Provider};

use super::{
    empty_tree_id,
    tree::{TreeIdentifier, TreeLeafNode},
    BasicIndexer, Error, IndexKey, IndexPath, IndexPathItem, RecursiveIndexer, Result,
    SearchResult, Tree, TreeNode, TreeReader,
};

/// A `StringPathIndexer` is an indexer that adds resources according to a
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
pub struct StringPathIndexer {
    /// The separator used to separate the path elements.
    #[serde(default = "StringPathIndexer::default_path_separator")]
    path_separator: char,

    /// Whether to keep empty directories when removing a resource.
    #[serde(default)]
    keep_empty_branches: bool,
}

impl Default for StringPathIndexer {
    fn default() -> Self {
        Self::new(Self::default_path_separator())
    }
}

impl StringPathIndexer {
    /// Create a new static indexer with the specified key size.
    ///
    /// By default, the indexer will use a minimum of 2 children and a maximum
    /// of 256 children per layer.
    pub fn new(path_separator: char) -> Self {
        Self {
            path_separator,
            keep_empty_branches: false,
        }
    }

    fn default_path_separator() -> char {
        '/'
    }

    /// Set whether to remove empty directories when removing a resource.
    pub fn set_keep_empty_branches(&mut self, keep_empty_branches: bool) {
        self.keep_empty_branches = keep_empty_branches;
    }

    /// Convert the index key to its UTF-8 string representation, and trims it
    /// to a conforming format that allows later splitting by
    /// `split_first_index_key` and `split_last_index_key`.
    ///
    /// # Errors
    ///
    /// If the index key is not UTF-8 encoded, an error is returned.
    fn sanitize_index_key<'k>(&self, index_key: &'k [u8]) -> Result<&'k str> {
        let index_key = std::str::from_utf8(index_key)
            .map_err(|err| Error::InvalidIndexKey(format!("invalid UTF-8: {}", err)))?;

        if !index_key.starts_with(self.path_separator) {
            return Err(Error::InvalidIndexKey(
                "index key must start with a '/' character".to_string(),
            ));
        }

        Ok(index_key.trim_end_matches(self.path_separator))
    }

    /// Split the index key into its first path element and the remaining path.
    ///
    /// The index key should have been sanitized before calling this function.
    /// See `sanitize_index_key`.
    fn split_first_index_key<'k>(&self, index_key: &'k str) -> (&'k str, &'k str) {
        match index_key[1..].find(self.path_separator) {
            Some(pos) => (&index_key[..=pos], &index_key[pos + 1..]),
            None => (index_key, ""),
        }
    }

    /// Split the index key into its optional folder hierarchy and "file name"
    /// part.
    ///
    /// The index key should have been sanitized before calling this function.
    /// See `sanitize_index_key`.
    fn split_last_index_key<'k>(&self, index_key: &'k str) -> (&'k str, &'k str) {
        match index_key[1..].rfind(self.path_separator) {
            Some(pos) => (&index_key[..=pos], &index_key[pos + 1..]),
            None => ("", index_key),
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
            let mut remaining_key: &str = self.sanitize_index_key(index_key)?;
            let mut local_key: &str;

            let mut stack = IndexPath::default();

            loop {
                stack.push(IndexPathItem {
                    tree: current_node.clone(),
                    key: remaining_key.as_bytes(),
                });

                (local_key, remaining_key) = self.split_first_index_key(remaining_key);

                match current_node.into_children(local_key.as_bytes()) {
                    None => return Ok(SearchResult::NotFound(stack)),
                    Some(node) => {
                        // We found a children with the local key: let's
                        // replace the key of the last element in the
                        // stack to reflect that.
                        stack.last_mut().unwrap().key = local_key.as_bytes();

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
}

#[async_trait]
impl BasicIndexer for StringPathIndexer {
    async fn get_leaf(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &IndexKey,
    ) -> Result<Option<TreeLeafNode>> {
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
                let mut node = TreeNode::Leaf(leaf_node);
                let mut item_key: &str;
                let mut local_key: &str;

                node = loop {
                    (item_key, local_key) =
                        self.split_last_index_key(self.sanitize_index_key(item.key)?);

                    if item_key.is_empty() {
                        break node;
                    }

                    item.key = item_key.as_bytes();

                    let tree = Tree {
                        count: 1,
                        total_size: size_delta,
                        children: vec![(local_key.into(), node)],
                    };

                    let tree_id = provider.write_tree(&tree).await?;

                    node = TreeNode::Branch(tree_id);
                };

                loop {
                    item.tree.count += 1;

                    if let Some(TreeNode::Branch(old_node)) =
                        item.tree.insert_children(item.key, node)
                    {
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

    async fn replace_leaf<'call>(
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
                        if let Some(TreeNode::Branch(old_node)) =
                            item.tree.insert_children(item.key, node)
                        {
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
        let root = provider.read_tree(root_id).await?;

        match self.search(provider, &root, index_key).await? {
            SearchResult::Leaf(mut stack, existing_leaf_node) => {
                let mut item = stack.pop().expect("stack is not empty");

                loop {
                    if let Some(TreeNode::Branch(old_node)) = item.tree.remove_children(item.key) {
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

                        return Ok((empty_tree_id(provider).await?, existing_leaf_node));
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

                    if let Some(TreeNode::Branch(old_node)) =
                        item.tree.insert_children(item.key, node)
                    {
                        provider.unwrite(old_node.as_identifier()).await?;
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
impl RecursiveIndexer for StringPathIndexer {
    async fn get(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &IndexKey,
    ) -> Result<Option<TreeNode>> {
        let root = provider.read_tree(root_id).await?;

        Ok(self.search(provider, &root, index_key).await?.into())
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
    fn test_string_path_indexer_sanitize() {
        let idx = StringPathIndexer::new('/');

        // Missing leading slash.
        let b = "foo/bar/baz/qux/quux";
        assert!(idx.sanitize_index_key(b.as_bytes()).is_err());

        let b = "/foo/bar/baz/qux/quux///";
        assert_eq!(
            idx.sanitize_index_key(b.as_bytes()).unwrap(),
            "/foo/bar/baz/qux/quux"
        );
    }

    #[test]
    fn test_string_path_indexer_split_first() {
        let idx = StringPathIndexer::new('/');

        let b = "/foo/bar/baz/qux/quux";
        assert_eq!(idx.split_first_index_key(b), ("/foo", "/bar/baz/qux/quux"));

        let b = "/foo-bar";
        assert_eq!(idx.split_first_index_key(b), ("/foo-bar", ""));
    }

    #[test]
    fn test_string_path_indexer_split_last() {
        let idx = StringPathIndexer::new('/');

        let b = "/foo/bar/baz/qux/quux";
        assert_eq!(idx.split_last_index_key(b), ("/foo/bar/baz/qux", "/quux"));

        let b = "/foo-bar";
        assert_eq!(idx.split_last_index_key(b), ("", "/foo-bar"));
    }

    #[tokio::test]
    async fn test_string_path_indexer() {
        let provider = Provider::new_in_memory();
        let idx = StringPathIndexer::default();

        // This is our starting point: we write an empty tree.
        //
        // In all likelyhood, the generated identifier will benefit from
        // small-content optimization and not actually be written anywhere.
        let tree_id = empty_tree_id(&provider).await.unwrap();

        assert_node_does_not_exist!(idx, provider, tree_id, "/fruits/pear.txt");

        let (tree_id, _) = add_leaf!(idx, provider, tree_id, "/fruits/apple.txt", "apple");
        let (tree_id, pear_node) = add_leaf!(idx, provider, tree_id, "/fruits/pear.txt", "pear");
        let (tree_id, _) = add_leaf!(idx, provider, tree_id, "/fruits/banana.txt", "banana");
        let (tree_id, _) = add_leaf!(idx, provider, tree_id, "/vegetables/tomato.txt", "tomato");

        // Uncomment this to generate GraphViz output for the above tree.
        //crate::indexing::GraphvizVisitor::create("tree.dot")
        //    .await
        //    .unwrap()
        //    .visit(&ct, &tree_id)
        //    .await
        //    .unwrap();

        let tree = read_tree!(provider, tree_id);
        assert_eq!(tree.count, 4);
        assert_eq!(tree.total_size(), 21);

        // Let's perform a search in the resulting tree. It should yield the leaf node.
        assert_leaf!(idx, provider, tree_id, "/fruits/pear.txt", pear_node);

        // Let's do the same for a branch.
        assert_branch!(idx, provider, tree_id, "/vegetables/");
        assert_branch!(idx, provider, tree_id, "/vegetables");

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

        // The only identifier that should be referenced is the root.
        let ids = provider.referenced().await;
        assert_eq!(&ids, &[tree_id.as_identifier().clone()]);
    }
}
