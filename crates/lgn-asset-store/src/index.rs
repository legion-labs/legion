use std::collections::VecDeque;

use async_stream::stream;
use lgn_content_store2::{ContentProvider, ContentReader};
use serde::{de::DeserializeOwned, Serialize};
use tokio_stream::Stream;

use crate::{Asset, Result, Tree, TreeNode};

/// An index of assets.
pub struct Index<Metadata> {
    key_path_splitter: KeyPathSplitter,
    key_getter: Box<dyn KeyGetter<Metadata, KeyType = String>>,
}

impl<Metadata> Index<Metadata>
where
    Metadata: Serialize + DeserializeOwned,
{
    pub fn new(
        key_path_splitter: KeyPathSplitter,
        key_getter: impl KeyGetter<Metadata, KeyType = String> + 'static,
    ) -> Self {
        Self {
            key_path_splitter,
            key_getter: Box::new(key_getter),
        }
    }

    /// Get an asset by its key.
    ///
    /// If no such asset exists, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset cannot be searched for.
    pub async fn get_asset(
        &self,
        provider: impl ContentReader + Send + Sync + Copy,
        tree: &Tree,
        key: &str,
    ) -> Result<Option<Asset<Metadata>>> {
        let path = self.key_path_splitter.split_key(key);

        if path.is_empty() {
            return Ok(None); // If the key is empty, the asset cannot be found.
        }

        let (asset_key, path) = path.split_last().unwrap();

        // This returns [tree, tree_node1, tree_node2, ..., tree_nodeN] where
        // tree_nodeN is the last node in the path which should contain the
        // asset.
        //
        // If N is less than the length of the path + 1, then the path is not
        // complete and new empty nodes are created.
        if let Some(tree) = self.resolve_tree(provider, tree, path).await? {
            tree.lookup_asset(provider, asset_key).await
        } else {
            Ok(None)
        }
    }

    /// Add an asset to the specified index tree.
    ///
    /// Any existing asset with the same key will be overwritten silently.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset could not be added.
    pub async fn add_asset(
        &self,
        provider: impl ContentProvider + Send + Sync + Copy,
        tree: Tree,
        asset: Asset<Metadata>,
    ) -> Result<Tree> {
        let key = match self.key_getter.get_key(asset.metadata()) {
            Some(key) => key,
            None => return Ok(tree), // The asset does not have the required key, and therefore cannot be added to the tree.
        };

        let path = self.key_path_splitter.split_key(&key);

        if path.is_empty() {
            return Ok(tree); // If the key is empty, assume the asset cannot be added to the tree.
        }

        let asset_id = asset.save(provider).await?;

        let (asset_key, path) = path.split_last().unwrap();

        // This returns [tree, tree_node1, tree_node2, ..., tree_nodeN] where
        // tree_nodeN is the last node in the path which should contain the
        // asset.
        //
        // If N is less than the length of the path + 1, then the path is not
        // complete and new empty nodes are created.
        let mut tree_path = self.resolve_tree_path(provider, tree, path).await?;

        while tree_path.len() < path.len() + 1 {
            tree_path.push(Tree::default());
        }

        // Let's create an iterator of [(asset_key, tree_nodeN), ..., (key, tree_node1)].
        let mut iter = path
            .iter()
            .chain(std::iter::once(asset_key))
            .rev()
            .zip(tree_path.into_iter().rev());

        let mut last_tree = iter
            .next()
            .map(|(key, tree)| tree.with_named_asset_id((*key).to_string(), asset_id))
            .unwrap();
        let mut last_tree_id = last_tree.save(provider).await?;

        for (key, tree) in iter {
            last_tree = tree.with_named_tree_id((*key).to_string(), last_tree_id);
            last_tree_id = last_tree.save(provider).await?;
        }

        Ok(last_tree)
    }

    /// Remove an asset from the specified index tree.
    ///
    /// If the asset is not found, the tree is returned unchanged.
    ///
    /// Empty tree nodes in the removal path are removed.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset could not be removed.
    pub async fn remove_asset(
        &self,
        provider: impl ContentProvider + Send + Sync + Copy,
        tree: Tree,
        asset: Asset<Metadata>,
    ) -> Result<Tree> {
        let key = match self.key_getter.get_key(asset.metadata()) {
            Some(key) => key,
            None => return Ok(tree), // The asset does not have the required key, and therefore cannot be added to the tree.
        };

        let path = self.key_path_splitter.split_key(&key);

        if path.is_empty() {
            return Ok(tree); // If the key, assume the asset cannot be added to the tree.
        }

        let (asset_key, path) = path.split_last().unwrap();

        let mut tree_path = self.resolve_tree_path(provider, tree, path).await?;

        if tree_path.len() < path.len() + 1 {
            // If the asset is not found, the tree is returned unchanged.
            return Ok(tree_path.swap_remove(0));
        }

        // Let's create an iterator of [(asset_key, tree_nodeN), ..., (key, tree_node1)].
        let mut iter = path
            .iter()
            .chain(std::iter::once(asset_key))
            .rev()
            .zip(tree_path.into_iter().rev());

        let mut last_tree = iter
            .next()
            .map(|(key, tree)| tree.without_child(key))
            .unwrap();

        for (key, tree) in iter {
            last_tree = if last_tree.is_empty() {
                tree.without_child(key)
            } else {
                tree.with_named_tree_id((*key).to_string(), last_tree.save(provider).await?)
            }
        }

        last_tree.save(provider).await?;

        Ok(last_tree)
    }

    /// Returns a stream that iterates over all assets in the specified tree.
    ///
    /// # Warning
    ///
    /// This method is not intended to be used in production as it iterates over
    /// the entire tree. If you think you need to use this method, please think
    /// twice, and then some more.
    pub fn all_assets<'s>(
        &'s self,
        provider: impl ContentReader + Send + Sync + Copy + 's,
        tree: Tree,
    ) -> impl Stream<Item = (String, Result<Asset<Metadata>>)> + 's {
        let mut trees = VecDeque::new();
        trees.push_back((String::default(), tree));

        stream! {
            while let Some((prefix, current_tree)) = trees.pop_front() {
                for (key, node) in current_tree.iter() {
                    let new_prefix = self.key_path_splitter.join_keys(&prefix, key);

                    match node {
                        TreeNode::Leaf(asset_id) => {
                            yield (new_prefix, Asset::<Metadata>::load(provider, asset_id).await);
                        },
                        TreeNode::Branch(tree_id) => {
                            match Tree::load(provider, tree_id).await {
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

    /// Resolve a tree from a path.
    ///
    /// Might be used to fetch a "directory" of assets.
    ///
    /// If the path does not exist, `Ok(None)` is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be resolved.
    pub async fn resolve_tree(
        &self,
        provider: impl ContentReader + Send + Sync + Copy,
        tree: &Tree,
        path: &[&str],
    ) -> Result<Option<Tree>> {
        if path.is_empty() {
            return Ok(None);
        }

        let (first, path) = path.split_first().unwrap();

        let mut tree = if let Some(node) = tree.lookup_tree(provider, first).await? {
            node
        } else {
            return Ok(None);
        };

        for element in path {
            if let Some(node) = tree.lookup_tree(provider, element).await? {
                tree = node;
            } else {
                return Ok(None);
            }
        }

        Ok(Some(tree.clone()))
    }

    /// Resolve a path of trees.
    async fn resolve_tree_path(
        &self,
        provider: impl ContentProvider + Send + Sync + Copy,
        tree: Tree,
        path: &[&str],
    ) -> Result<Vec<Tree>> {
        let mut result = Vec::with_capacity(path.len());
        result.push(tree);

        for element in path {
            if let Some(node) = result
                .last()
                .unwrap()
                .lookup_tree(provider, element)
                .await?
            {
                result.push(node);
            } else {
                break;
            }
        }

        Ok(result)
    }
}

/// A trait for getting a key from metadata.
pub trait KeyGetter<Metadata> {
    type KeyType;

    fn get_key(&self, metadata: &Metadata) -> Option<Self::KeyType>;
}

/// A blanket implementation of `KeyGetter` for functions.
impl<Metadata, T> KeyGetter<Metadata> for T
where
    T: Fn(&Metadata) -> Option<String>,
{
    type KeyType = String;

    fn get_key(&self, metadata: &Metadata) -> Option<Self::KeyType> {
        (self)(metadata)
    }
}

/// Split string keys into a path.
///
/// Path segments are never empty.
pub enum KeyPathSplitter {
    Separator(char),
    Size(usize),
}

impl KeyPathSplitter {
    /// Create a new key path splitter that uses the specified separator.
    fn split_key<'k>(&self, mut key: &'k str) -> Vec<&'k str> {
        if key.is_empty() {
            return vec![];
        }

        match self {
            KeyPathSplitter::Separator(separator) => {
                // Prefix and suffix separators are removed, always.
                key.trim_matches(*separator)
                    .split(*separator)
                    .filter(|s| !s.is_empty())
                    .collect()
            }

            KeyPathSplitter::Size(size) => {
                assert!(*size > 0, "size must be strictly positive");

                let mut res = Vec::with_capacity(1 + (key.len() - 1) / size);

                while !key.is_empty() {
                    if key.len() > *size {
                        let (head, tail) = key.split_at(*size);
                        res.push(head);
                        key = tail;
                    } else {
                        res.push(key);
                        break;
                    }
                }

                res
            }
        }
    }

    fn join_keys(&self, a: &str, b: &str) -> String {
        match self {
            KeyPathSplitter::Separator(separator) => format!("{}{}{}", a, separator, b),
            KeyPathSplitter::Size(_) => format!("{}{}", a, b),
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::stream::StreamExt;
    use lgn_content_store2::MemoryProvider;
    use serde::{Deserialize, Serialize};

    use crate::{Asset, Index, KeyPathSplitter, Tree};

    #[test]
    fn test_key_path_splitter_separator() {
        let splitter = KeyPathSplitter::Separator('/');
        assert_eq!(splitter.split_key(""), Vec::<&str>::new());
        assert_eq!(splitter.split_key("/"), Vec::<&str>::new());
        assert_eq!(splitter.split_key("/a"), vec!["a"]);
        assert_eq!(splitter.split_key("/a/b"), vec!["a", "b"]);
        assert_eq!(splitter.split_key("/a/b/c/"), vec!["a", "b", "c"]);
        assert_eq!(splitter.split_key("a/b/c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_key_path_splitter_size() {
        let splitter = KeyPathSplitter::Size(2);
        assert_eq!(splitter.split_key(""), Vec::<&str>::new());
        assert_eq!(splitter.split_key("a"), vec!["a"]);
        assert_eq!(splitter.split_key("ab"), vec!["ab"]);
        assert_eq!(splitter.split_key("abc"), vec!["ab", "c"]);
        assert_eq!(splitter.split_key("abcd"), vec!["ab", "cd"]);
        assert_eq!(splitter.split_key("abcde"), vec!["ab", "cd", "e"]);
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Metadata {
        path: String,
        oid: String,
    }

    fn meta(path: &str, oid: &str) -> Metadata {
        Metadata {
            path: path.to_string(),
            oid: oid.to_string(),
        }
    }

    #[tokio::test]
    async fn test_index() {
        // In a real case obviously, we can use the default provider.
        let provider = &MemoryProvider::new();

        // Let's create an index that stores assets according to their path, and
        // splits the path for each '/'.
        let file_index = Index::new(KeyPathSplitter::Separator('/'), |m: &Metadata| {
            Some(m.path.clone())
        });
        // Let's create an index that stores assets according to their OID, and
        // splits the path for each 2 characters.
        let oid_index = Index::new(KeyPathSplitter::Size(2), |m: &Metadata| Some(m.oid.clone()));

        // Let's create a bunch of assets.
        let asset_a =
            Asset::new_from_data(provider, meta("/assets/a", "abcdef"), b"hello world from A")
                .await
                .unwrap();
        let asset_b =
            Asset::new_from_data(provider, meta("/assets/b", "abefef"), b"hello world from B")
                .await
                .unwrap();

        // We add each asset to both indexes.
        //
        // Note that the actual storage only happens once, thanks to the content
        // store implicit deduplication.
        let file_tree = file_index
            .add_asset(provider, Tree::default(), asset_a.clone())
            .await
            .unwrap();
        let oid_tree = oid_index
            .add_asset(provider, Tree::default(), asset_a.clone())
            .await
            .unwrap();
        let file_tree = file_index
            .add_asset(provider, file_tree, asset_b.clone())
            .await
            .unwrap();
        let oid_tree = oid_index
            .add_asset(provider, oid_tree, asset_b.clone())
            .await
            .unwrap();

        // This is how we can query assets.
        //
        // Note how we need three things:
        // - The index to query, which almost never changes across commits.
        // - The key that is indexed.
        // - The matching tree to query, which will likely be different for each commit.
        //
        // In a nutshell: the key is the 'what', the tree is the 'where' and the
        // index is the 'how'.
        assert_eq!(
            file_index
                .get_asset(provider, &file_tree, "/assets/a")
                .await
                .unwrap(),
            Some(asset_a.clone())
        );
        assert_eq!(
            oid_index
                .get_asset(provider, &oid_tree, "abcdef")
                .await
                .unwrap(),
            Some(asset_a.clone())
        );

        // Fetching by OID in the file index? No. Won't work, as expected.
        assert_eq!(
            file_index
                .get_asset(provider, &file_tree, "abcdef")
                .await
                .unwrap(),
            None,
        );

        // List all the assets in the index. Should be discouraged in real code: mostly useful for tests.
        let assets_as_files = file_index
            .all_assets(provider, file_tree)
            .map(|(key, asset)| (key, asset.unwrap()))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(
            assets_as_files,
            vec![
                ("/assets/a".to_string(), asset_a.clone()),
                ("/assets/b".to_string(), asset_b.clone())
            ]
        );

        let assets_as_oids = oid_index
            .all_assets(provider, oid_tree)
            .map(|(key, asset)| (key, asset.unwrap()))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(
            assets_as_oids,
            vec![
                ("abcdef".to_string(), asset_a.clone()),
                ("abefef".to_string(), asset_b.clone())
            ]
        );
    }
}
