use std::collections::{BTreeSet, VecDeque};

use crate::{ContentProvider, ContentReader, ContentWriterExt, Result};
use async_stream::stream;
use futures_util::pin_mut;
use serde::{de::DeserializeOwned, Serialize};
use tokio_stream::{Stream, StreamExt};

use super::{MultiResourcesTree, Resource, ResourceIdentifier, Tree, TreeNode, UniqueResourceTree};

/// An index of resources.
pub struct Index<Metadata, KeyType> {
    key_path_splitter: KeyPathSplitter,
    key_getter: Box<dyn KeyGetter<Metadata, KeyType = KeyType>>,
}

impl<Metadata> Index<Metadata, String>
where
    Metadata: Serialize + DeserializeOwned,
{
    /// Add an resource to the specified unique resource tree.
    ///
    /// Any existing resource with the same key will be overwritten silently.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource could not be added.
    pub async fn add_resource(
        &self,
        provider: impl ContentProvider + Send + Sync,
        tree: UniqueResourceTree,
        resource: &Resource<Metadata>,
    ) -> Result<UniqueResourceTree> {
        let resource_id = provider
            .write_content(&resource.as_vec())
            .await
            .map(ResourceIdentifier)?;

        let key = match self.key_getter.get_key(resource.metadata()) {
            Some(key) => key,
            None => return Ok(tree), // The resource does not have the required key, and therefore cannot be added to the tree.
        };

        self.add_entry(provider, tree, resource_id, &key).await
    }

    /// Remove an resource from the specified tree.
    ///
    /// If the resource is not found, the tree is returned unchanged.
    ///
    /// Empty tree nodes in the removal path are removed too.
    ///
    /// # Errors
    ///
    /// Returns an error if the entry could not be removed.
    pub async fn remove_resource(
        &self,
        provider: impl ContentProvider + Send + Sync,
        tree: UniqueResourceTree,
        resource: &Resource<Metadata>,
    ) -> Result<UniqueResourceTree> {
        let key = match self.key_getter.get_key(resource.metadata()) {
            Some(key) => key,
            None => return Ok(tree), // The resource does not have the required key, and therefore cannot be removed from the tree.
        };

        self.remove_entry(provider, tree, &key).await
    }

    /// Get an resource by its key in a unique resource tree.
    ///
    /// If no such resource exists, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource cannot be searched for.
    pub async fn get_resource(
        &self,
        provider: impl ContentReader + Send + Sync,
        tree: &UniqueResourceTree,
        key: &str,
    ) -> Result<Option<Resource<Metadata>>> {
        match self.get_entry(&provider, tree, key).await? {
            Some(resource_id) => Resource::load(provider, &resource_id).await.map(Some),
            None => Ok(None),
        }
    }

    /// Returns a stream that iterates over all resources in the specified unique
    /// resource tree.
    ///
    /// # Warning
    ///
    /// This method is not intended to be used in production as it iterates over
    /// the entire tree. If you think you need to use this method, please think
    /// twice, and then some more.
    pub fn all_resources<'s>(
        &'s self,
        provider: impl ContentReader + Send + Sync + 's,
        tree: UniqueResourceTree,
    ) -> impl Stream<Item = (String, Result<Resource<Metadata>>)> + 's {
        stream! {
            let resource_ids = self.all_entries(&provider, tree);

            pin_mut!(resource_ids); // needed for iteration

            while let Some((prefix, resource_id)) = resource_ids.next().await {
                match resource_id {
                    Ok(resource_id) => yield (prefix, Resource::<Metadata>::load(&provider, &resource_id).await),
                    Err(err) => yield (prefix, Err(err)),
                }
            }
        }
    }
}

impl<Metadata> Index<Metadata, BTreeSet<String>>
where
    Metadata: Serialize + DeserializeOwned,
{
    /// Add an resource to the specified multi resources tree.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource could not be added.
    pub async fn add_resource(
        &self,
        provider: impl ContentProvider + Send + Sync,
        mut tree: MultiResourcesTree,
        resource: &Resource<Metadata>,
    ) -> Result<MultiResourcesTree> {
        let resource_id = provider
            .write_content(&resource.as_vec())
            .await
            .map(ResourceIdentifier)?;

        let keys = match self.key_getter.get_key(resource.metadata()) {
            Some(key) => key,
            None => return Ok(tree), // The resource does not have the required key, and therefore cannot be added to the tree.
        };

        for key in &keys {
            let resource_ids = match self.get_entry(&provider, &tree, key).await? {
                Some(mut resource_ids) => {
                    resource_ids.insert(resource_id.clone());
                    resource_ids
                }
                None => BTreeSet::from_iter([resource_id.clone()]),
            };

            tree = self.add_entry(&provider, tree, resource_ids, key).await?;
        }

        Ok(tree)
    }

    /// Remove an resource from the specified multi resources tree.
    ///
    /// If the resource is not found, the tree is returned unchanged.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource could not be removed.
    pub async fn remove_resource(
        &self,
        provider: impl ContentProvider + Send + Sync,
        mut tree: MultiResourcesTree,
        resource: &Resource<Metadata>,
    ) -> Result<MultiResourcesTree> {
        let resource_id = provider
            .write_content(&resource.as_vec())
            .await
            .map(ResourceIdentifier)?;

        let keys = match self.key_getter.get_key(resource.metadata()) {
            Some(key) => key,
            None => return Ok(tree), // The resource does not have the required key, and therefore cannot be added to the tree.
        };

        for key in &keys {
            if let Some(mut resource_ids) = self.get_entry(&provider, &tree, key).await? {
                // Only actually write if the resource was listed.
                if resource_ids.remove(&resource_id) {
                    if resource_ids.is_empty() {
                        tree = self.remove_entry(&provider, tree, key).await?;
                    } else {
                        tree = self.add_entry(&provider, tree, resource_ids, key).await?;
                    }
                }
            }
        }

        Ok(tree)
    }

    /// Get a stream of resources by their key in a multi resources tree.
    ///
    /// If no such resource exists, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource cannot be searched for.
    pub async fn get_resources<'s>(
        &'s self,
        provider: impl ContentReader + Send + Sync + 's,
        tree: &'s MultiResourcesTree,
        key: &str,
    ) -> Result<Option<impl Stream<Item = Result<Resource<Metadata>>> + 's>> {
        Ok(self
            .get_entry(&provider, tree, key)
            .await?
            .map(|resource_ids| {
                stream! {
                    for resource_id in resource_ids {
                        yield Resource::load(&provider, &resource_id).await;
                    }
                }
            }))
    }
}

impl<Metadata, KeyType> Index<Metadata, KeyType>
where
    Metadata: Serialize + DeserializeOwned,
{
    pub fn new(
        key_path_splitter: KeyPathSplitter,
        key_getter: impl KeyGetter<Metadata, KeyType = KeyType> + 'static,
    ) -> Self {
        Self {
            key_path_splitter,
            key_getter: Box::new(key_getter),
        }
    }

    /// Get a leaf by its key in a tree.
    ///
    /// If no such leaf exists, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the leaf cannot be searched for.
    pub async fn get_entry<LeafType>(
        &self,
        provider: impl ContentReader + Send + Sync,
        tree: &Tree<LeafType>,
        key: &str,
    ) -> Result<Option<LeafType>>
    where
        LeafType: Clone + DeserializeOwned,
    {
        let path = self.key_path_splitter.split_key(key);

        if path.is_empty() {
            return Ok(None); // If the key is empty, the resource cannot be found.
        }

        let (leaf_key, path) = path.split_last().unwrap();

        // This returns [tree, tree_node1, tree_node2, ..., tree_nodeN] where
        // tree_nodeN is the last node in the path which should contain the
        // resource.
        //
        // If N is less than the length of the path + 1, then the path is not
        // complete and new empty nodes are created.
        if let Some(tree) = self.resolve_tree_from_path(provider, tree, path).await? {
            Ok(tree.lookup_leaf(leaf_key).cloned())
        } else {
            Ok(None)
        }
    }

    /// Add an entry to the specified tree.
    ///
    /// Any existing entry with the same key will be overwritten silently.
    ///
    /// # Errors
    ///
    /// Returns an error if the entry could not be added.
    pub async fn add_entry<LeafType>(
        &self,
        provider: impl ContentProvider + Send + Sync,
        tree: Tree<LeafType>,
        entry: LeafType,
        key: &str,
    ) -> Result<Tree<LeafType>>
    where
        LeafType: DeserializeOwned + Serialize,
    {
        let path = self.key_path_splitter.split_key(key);

        if path.is_empty() {
            return Ok(tree); // If the key is empty, assume the entry cannot be added to the tree.
        }

        let (resource_key, path) = path.split_last().unwrap();

        // This returns [tree, tree_node1, tree_node2, ..., tree_nodeN] where
        // tree_nodeN is the last node in the path which should contain the
        // entry.
        //
        // If N is less than the length of the path + 1, then the path is not
        // complete and new empty nodes are created.
        let mut tree_path = self.resolve_tree_path(&provider, tree, path).await?;

        while tree_path.len() < path.len() + 1 {
            tree_path.push(Tree::default());
        }

        // Let's create an iterator of [(resource_key, tree_nodeN), ..., (key, tree_node1)].
        let mut iter = path
            .iter()
            .chain(std::iter::once(resource_key))
            .rev()
            .zip(tree_path.into_iter().rev());

        let mut last_tree = iter
            .next()
            .map(|(key, tree)| tree.with_named_leaf((*key).to_string(), entry))
            .unwrap();
        let mut last_tree_id = last_tree.save(&provider).await?;

        for (key, tree) in iter {
            last_tree = tree.with_named_branch((*key).to_string(), last_tree_id);
            last_tree_id = last_tree.save(&provider).await?;
        }

        Ok(last_tree)
    }

    /// Remove an entry from the specified tree.
    ///
    /// If the entry is not found, the tree is returned unchanged.
    ///
    /// Empty tree nodes in the removal path are removed too.
    ///
    /// # Errors
    ///
    /// Returns an error if the entry could not be removed.
    pub async fn remove_entry<LeafType>(
        &self,
        provider: impl ContentProvider + Send + Sync,
        tree: Tree<LeafType>,
        key: &str,
    ) -> Result<Tree<LeafType>>
    where
        LeafType: DeserializeOwned + Serialize,
    {
        let path = self.key_path_splitter.split_key(key);

        if path.is_empty() {
            return Ok(tree); // If the key, assume the resource cannot be added to the tree.
        }

        let (resource_key, path) = path.split_last().unwrap();

        let mut tree_path = self.resolve_tree_path(&provider, tree, path).await?;

        if tree_path.len() < path.len() + 1 {
            // If the resource is not found, the tree is returned unchanged.
            return Ok(tree_path.swap_remove(0));
        }

        // Let's create an iterator of [(resource_key, tree_nodeN), ..., (key, tree_node1)].
        let mut iter = path
            .iter()
            .chain(std::iter::once(resource_key))
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
                tree.with_named_branch((*key).to_string(), last_tree.save(&provider).await?)
            };
        }

        last_tree.save(&provider).await?;

        Ok(last_tree)
    }

    /// Returns a stream that iterates over all entries in the specified tree.
    ///
    /// # Warning
    ///
    /// This method is not intended to be used in production as it iterates over
    /// the entire tree. If you think you need to use this method, please think
    /// twice, and then some more.
    pub fn all_entries<'s, LeafType>(
        &'s self,
        provider: impl ContentReader + Send + Sync + 's,
        tree: Tree<LeafType>,
    ) -> impl Stream<Item = (String, Result<LeafType>)> + 's
    where
        LeafType: Clone + DeserializeOwned + 's,
    {
        let mut trees = VecDeque::new();
        trees.push_back((String::default(), tree));

        stream! {
            while let Some((prefix, current_tree)) = trees.pop_front() {
                for (key, node) in current_tree.iter() {
                    let new_prefix = self.key_path_splitter.join_keys(&prefix, key);

                    match node {
                        TreeNode::Leaf(entry) => {
                            yield (new_prefix, Ok(entry.clone()));
                        },
                        TreeNode::Branch(tree_id) => {
                            match Tree::load(&provider, tree_id).await {
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
    /// Might be used to fetch a "directory" of resources.
    ///
    /// If the path does not exist, `Ok(None)` is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be resolved.
    pub async fn resolve_tree<LeafType>(
        &self,
        provider: impl ContentReader + Send + Sync,
        tree: &Tree<LeafType>,
        key: &str,
    ) -> Result<Option<Tree<LeafType>>>
    where
        LeafType: DeserializeOwned + Clone,
    {
        let path = self.key_path_splitter.split_key(key);

        self.resolve_tree_from_path(provider, tree, &path).await
    }

    /// Resolve a tree from a path.
    ///
    /// Might be used to fetch a "directory" of resources.
    ///
    /// If the path does not exist, `Ok(None)` is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be resolved.
    async fn resolve_tree_from_path<LeafType>(
        &self,
        provider: impl ContentReader + Send + Sync,
        tree: &Tree<LeafType>,
        path: &[&str],
    ) -> Result<Option<Tree<LeafType>>>
    where
        LeafType: DeserializeOwned + Clone,
    {
        if path.is_empty() {
            return Ok(None); // If the key is empty, there is nothing to resolve.
        }

        let (first, path) = path.split_first().unwrap();

        let mut tree = if let Some(node) = tree.lookup_branch(&provider, first).await? {
            node
        } else {
            return Ok(None);
        };

        for element in path {
            if let Some(node) = tree.lookup_branch(&provider, element).await? {
                tree = node;
            } else {
                return Ok(None);
            }
        }

        Ok(Some(tree.clone()))
    }

    /// Resolve a path of trees.
    async fn resolve_tree_path<LeafType>(
        &self,
        provider: impl ContentProvider + Send + Sync,
        tree: Tree<LeafType>,
        path: &[&str],
    ) -> Result<Vec<Tree<LeafType>>>
    where
        LeafType: DeserializeOwned,
    {
        let mut result = Vec::with_capacity(path.len());
        result.push(tree);

        for element in path {
            if let Some(node) = result
                .last()
                .unwrap()
                .lookup_branch(&provider, element)
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
impl<Metadata, KeyType, T> KeyGetter<Metadata> for T
where
    T: Fn(&Metadata) -> Option<KeyType>,
{
    type KeyType = KeyType;

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
    use std::collections::BTreeSet;

    use crate::MemoryProvider;
    use futures_util::stream::StreamExt;
    use serde::{Deserialize, Serialize};

    use super::{Index, KeyPathSplitter, MultiResourcesTree, Resource, UniqueResourceTree};

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

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    struct Metadata {
        path: String,
        oid: String,
        parents: Vec<String>,
    }

    fn meta(path: &str, oid: &str) -> Metadata {
        Metadata {
            path: path.to_string(),
            oid: oid.to_string(),
            parents: Vec::new(),
        }
    }

    fn metap(path: &str, oid: &str, parents: &[&str]) -> Metadata {
        Metadata {
            path: path.to_string(),
            oid: oid.to_string(),
            parents: parents.iter().copied().map(ToOwned::to_owned).collect(),
        }
    }

    #[tokio::test]
    async fn test_unique_index() {
        // In a real case obviously, we can use the default provider.
        let provider = &MemoryProvider::new();

        // Let's create an index that stores resources according to their path, and
        // splits the path for each '/'.
        let file_index = Index::new(KeyPathSplitter::Separator('/'), |m: &Metadata| {
            Some(m.path.clone())
        });
        // Let's create an index that stores resources according to their OID, and
        // splits the path for each 2 characters.
        let oid_index = Index::new(KeyPathSplitter::Size(2), |m: &Metadata| Some(m.oid.clone()));

        // Let's create a bunch of resources.
        let resource_a = Resource::new_from_data(
            provider,
            meta("/resources/a", "abcdef"),
            b"hello world from A",
        )
        .await
        .unwrap();
        let resource_b = Resource::new_from_data(
            provider,
            meta("/resources/b", "abefef"),
            b"hello world from B",
        )
        .await
        .unwrap();

        // We add each resource to both indexes.
        //
        // Note that the actual storage only happens once, thanks to the content
        // store implicit deduplication.
        let file_tree = file_index
            .add_resource(provider, UniqueResourceTree::default(), &resource_a)
            .await
            .unwrap();
        let oid_tree = oid_index
            .add_resource(provider, UniqueResourceTree::default(), &resource_a)
            .await
            .unwrap();
        let file_tree = file_index
            .add_resource(provider, file_tree, &resource_b)
            .await
            .unwrap();
        let oid_tree = oid_index
            .add_resource(provider, oid_tree, &resource_b)
            .await
            .unwrap();

        // This is how we can query resources.
        //
        // Note how we need three things:
        // - The index to query, which almost never changes across commits.
        // - The key that is indexed.
        // - The matching tree to query, which will likely be different for each commit.
        //
        // In a nutshell: the key is the 'what', the tree is the 'where' and the
        // index is the 'how'.

        // Fetch an resource by path: use the file index.
        let resource = file_index
            .get_resource(provider, &file_tree, "/resources/a")
            .await
            .unwrap() // Result
            .unwrap(); // Option

        assert_eq!(resource, resource_a.clone());

        // We fetched that resource by its path: we can access any of its metadata!
        assert_eq!(resource.metadata().path, "/resources/a");
        assert_eq!(resource.metadata().oid, "abcdef");

        // Fetch an resource by OID: use the oid index.
        let resource = oid_index
            .get_resource(provider, &oid_tree, "abcdef")
            .await
            .unwrap() // Result
            .unwrap(); // Option

        assert_eq!(resource, resource_a.clone());

        // We fetched that resource by its OID: we can access any of its metadata!
        assert_eq!(resource.metadata().path, "/resources/a");
        assert_eq!(resource.metadata().oid, "abcdef");

        // Fetching by OID in the file index? No. Won't work, as expected.
        assert_eq!(
            file_index
                .get_resource(provider, &file_tree, "abcdef")
                .await
                .unwrap(),
            None,
        );

        // List all the resources in the index. Should be discouraged in real code: mostly useful for tests.
        let resources_as_files = file_index
            .all_resources(provider, file_tree.clone())
            .map(|(key, resource)| (key, resource.unwrap()))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(
            resources_as_files,
            vec![
                ("/resources/a".to_string(), resource_a.clone()),
                ("/resources/b".to_string(), resource_b.clone())
            ]
        );

        // The same with the OID index.
        let resources_as_oids = oid_index
            .all_resources(provider, oid_tree.clone())
            .map(|(key, resource)| (key, resource.unwrap()))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(
            resources_as_oids,
            vec![
                ("abcdef".to_string(), resource_a.clone()),
                ("abefef".to_string(), resource_b.clone())
            ]
        );

        // Remove an resource from the indexes.
        let file_tree = file_index
            .remove_resource(provider, file_tree, &resource_b)
            .await
            .unwrap();
        let oid_tree = oid_index
            .remove_resource(provider, oid_tree, &resource_b)
            .await
            .unwrap();

        // List all the resources in the index. Should be discouraged in real code: mostly useful for tests.
        let resources_as_files = file_index
            .all_resources(provider, file_tree.clone())
            .map(|(key, resource)| (key, resource.unwrap()))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(
            resources_as_files,
            vec![("/resources/a".to_string(), resource_a.clone()),]
        );

        // The same with the OID index.
        let resources_as_oids = oid_index
            .all_resources(provider, oid_tree.clone())
            .map(|(key, resource)| (key, resource.unwrap()))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(
            resources_as_oids,
            vec![("abcdef".to_string(), resource_a.clone()),]
        );
    }

    #[tokio::test]
    async fn test_multi_index() {
        // In a real case obviously, we can use the default provider.
        let provider = &MemoryProvider::new();

        // Let's create an index that stores resources according to their parents
        // and allows a reversed-dependency search.
        let deps_index = Index::new(KeyPathSplitter::Size(2), |m: &Metadata| {
            Some(m.parents.clone().into_iter().collect::<BTreeSet<_>>())
        });

        // Let's create a bunch of resources.
        let resource_a = Resource::new_from_data(
            provider,
            meta("/resources/a", "001122aa"),
            b"hello world from A",
        )
        .await
        .unwrap();
        let resource_b = Resource::new_from_data(
            provider,
            meta("/resources/b", "001122bb"),
            b"hello world from B",
        )
        .await
        .unwrap();
        // resource C has A and B for parents.
        let resource_c = Resource::new_from_data(
            provider,
            metap(
                "/resources/c",
                "001133cc",
                &[&resource_a.metadata().oid, &resource_b.metadata().oid],
            ),
            b"hello world from C",
        )
        .await
        .unwrap();
        // resource D has A and C for parents.
        let resource_d = Resource::new_from_data(
            provider,
            metap(
                "/resources/d",
                "002233dd",
                &[&resource_a.metadata().oid, &resource_c.metadata().oid],
            ),
            b"hello world from C",
        )
        .await
        .unwrap();

        // We add each resource to the index.
        let mut deps_tree = MultiResourcesTree::default();

        for resource in [&resource_a, &resource_b, &resource_c, &resource_d] {
            deps_tree = deps_index
                .add_resource(provider, deps_tree, resource)
                .await
                .unwrap();
        }

        // Get all the resources that depend on A.
        let resources = deps_index
            .get_resources(provider, &deps_tree, &resource_a.metadata().oid)
            .await
            .unwrap() // Result
            .unwrap() // Option
            .map(std::result::Result::unwrap)
            .collect::<BTreeSet<_>>()
            .await;

        // The order of returned resources is not specified (but if you are
        // curious: it actually depends on the ordering of resource identifiers
        // which are hashes).
        //
        // This should never matter, as there is no logical ordering of resources
        // dependencies.
        assert_eq!(
            resources,
            vec![resource_c.clone(), resource_d.clone()]
                .into_iter()
                .collect()
        );

        let resource_ids = deps_index
            .all_entries(provider, deps_tree.clone())
            .map(|(key, resource_ids)| (key, resource_ids.unwrap()))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(
            resource_ids,
            vec![
                (
                    resource_a.metadata().oid.clone(),
                    vec![resource_c.as_identifier(), resource_d.as_identifier()]
                        .into_iter()
                        .collect()
                ),
                (
                    resource_b.metadata().oid.clone(),
                    vec![resource_c.as_identifier()].into_iter().collect()
                ),
                (
                    resource_c.metadata().oid.clone(),
                    vec![resource_d.as_identifier()].into_iter().collect()
                ),
            ],
        );

        deps_tree = deps_index
            .remove_resource(provider, deps_tree, &resource_c)
            .await
            .unwrap();

        let resource_ids = deps_index
            .all_entries(provider, deps_tree.clone())
            .map(|(key, resource_ids)| (key, resource_ids.unwrap()))
            .collect::<Vec<_>>()
            .await;

        // If you look closely, you will notice than even though C was removed
        // from the index, it still appears as a parent for D.
        //
        // This is *NOT* a bug, as D effectively still references C.
        //
        // A proper integration of the resource store should of course query the
        // dependency index and update all related resources as well to avoid this
        // situation.
        assert_eq!(
            resource_ids,
            vec![
                (
                    resource_a.metadata().oid.clone(),
                    vec![resource_d.as_identifier()].into_iter().collect()
                ),
                (
                    resource_c.metadata().oid.clone(),
                    vec![resource_d.as_identifier()].into_iter().collect()
                ),
            ],
        );
    }
}
