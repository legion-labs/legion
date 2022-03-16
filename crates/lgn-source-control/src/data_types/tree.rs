use async_recursion::async_recursion;
use lgn_content_store2::{ChunkIdentifier, Chunker, ContentProvider};
use std::{
    cmp::Ordering,
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::{CanonicalPath, Change, ChangeType, Error, MapOtherError, Result, WithParentName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree {
    Directory {
        name: String,
        children: BTreeMap<String, Self>,
    },
    File {
        name: String,
        chunk_id: ChunkIdentifier,
    },
}

impl From<Tree> for lgn_source_control_proto::Tree {
    fn from(tree: Tree) -> Self {
        match tree {
            Tree::Directory { name, children } => Self {
                name,
                children: children.into_values().map(Into::into).collect(),
                chunk_id: "".to_string(),
            },
            Tree::File { name, chunk_id } => Self {
                name,
                children: vec![],
                chunk_id: chunk_id.to_string(),
            },
        }
    }
}

impl TryFrom<lgn_source_control_proto::Tree> for Tree {
    type Error = Error;

    fn try_from(tree: lgn_source_control_proto::Tree) -> Result<Self> {
        Ok(if tree.chunk_id.is_empty() {
            Self::Directory {
                name: tree.name,
                children: tree
                    .children
                    .into_iter()
                    .map(|n| {
                        let n: Result<Self> = n.try_into();
                        n.map(|n| (n.name().to_string(), n))
                    })
                    .collect::<Result<_>>()?,
            }
        } else {
            Self::File {
                name: tree.name,
                chunk_id: tree
                    .chunk_id
                    .parse()
                    .map_other_err("failed to parse chunk identifier")?,
            }
        })
    }
}

impl Ord for Tree {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Tree::Directory { name: a, .. }, Tree::Directory { name: b, .. })
            | (Tree::File { name: a, .. }, Tree::File { name: b, .. }) => a.cmp(b),
            (Tree::Directory { .. }, Tree::File { .. }) => Ordering::Less,
            (Tree::File { .. }, Tree::Directory { .. }) => Ordering::Greater,
        }
    }
}

impl PartialOrd for Tree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Tree {
    pub fn empty() -> Self {
        Self::Directory {
            name: "".to_string(),
            children: BTreeMap::new(),
        }
    }

    pub fn directory(name: String, children: impl IntoIterator<Item = Self>) -> Self {
        Self::Directory {
            name,
            children: children
                .into_iter()
                .map(|n| (n.name().to_string(), n))
                .collect(),
        }
    }

    pub fn file(name: String, chunk_id: ChunkIdentifier) -> Self {
        Self::File { name, chunk_id }
    }

    /// Creates a tree from an existing root directory.
    ///
    /// # Arguments
    ///
    /// The specified root directory must exist.
    ///
    /// The `ignore_list` is a list of canonical paths, relative to the
    /// specified root.
    ///
    /// Any path that is in the `ignore_list` or whose parent directory is
    /// ignored, will be ignored.
    ///
    /// All the files and subdirectories are considered.
    ///
    /// If the directory contains a symbolic link, an error is returned.
    ///
    /// # Returns
    ///
    /// A complete tree, with the root directory as the root.
    pub async fn from_root(
        chunker: &Chunker<impl ContentProvider + Send + Sync>,
        root: impl AsRef<Path>,
        filter: &TreeFilter,
    ) -> Result<Self> {
        let root = tokio::fs::canonicalize(root)
            .await
            .map_other_err("failed to make the root path canonical")?;

        Self::from_path(chunker, &root, &root, filter).await
    }

    /// Creates a tree from an existing absolute and canonicalized root directory.
    #[async_recursion]
    async fn from_path(
        chunker: &Chunker<impl ContentProvider + Send + Sync>,
        root: &Path,
        path: &Path,
        filter: &TreeFilter,
    ) -> Result<Self> {
        let cpath = CanonicalPath::new_from_canonical_root(root, path).await?;

        filter.check(&cpath)?;

        let path = cpath.to_path_buf(root);

        if path.is_file() {
            return Self::from_file_path(chunker, &path).await;
        }

        let name = cpath.name().unwrap_or_default();

        let mut entries = tokio::fs::read_dir(&path)
            .await
            .map_other_err(format!("failed to read directory `{}`", path.display()))?;

        let mut children = Vec::new();

        while let Some(sub_path) = entries.next_entry().await.map_other_err(format!(
            "failed to iterate over directory `{}`",
            path.display()
        ))? {
            match Self::from_path(chunker, root, &sub_path.path(), filter).await {
                Ok(child) => {
                    children.push(child);
                }
                Err(Error::PathExcluded { .. } | Error::PathNotIncluded { .. }) => {}
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(Self::directory(name.to_string(), children))
    }

    async fn from_file_path(
        chunker: &Chunker<impl ContentProvider + Send + Sync>,
        path: &Path,
    ) -> Result<Self> {
        let name = path
            .file_name()
            .ok_or_else(|| Error::Other {
                context: format!("failed to get file name from path `{}`", path.display()),
                source: anyhow::anyhow!("path does not have a file name"),
            })?
            .to_string_lossy();

        let contents = tokio::fs::read(&path)
            .await
            .map_other_err(format!("failed to read `{}`", path.display()))?;

        let chunk_id = chunker.get_chunk_identifier(&contents);

        Ok(Self::file(name.to_string(), chunk_id))
    }

    pub fn filter(&self, filter: &TreeFilter) -> Result<Self> {
        self.filter_with_root(filter, &CanonicalPath::root())
    }

    fn filter_with_root(&self, filter: &TreeFilter, root: &CanonicalPath) -> Result<Self> {
        filter.check(root)?;

        Ok(match self {
            Self::File { .. } => self.clone(),
            Self::Directory { name, children } => {
                let children = children
                    .iter()
                    .filter_map(|(name, child)| {
                        match child.filter_with_root(filter, &root.clone().append(name)) {
                            Ok(child) => Some(Ok(child)),
                            Err(Error::PathExcluded { .. } | Error::PathNotIncluded { .. }) => None,
                            Err(err) => Some(Err(err)),
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;

                Self::directory(name.clone(), children)
            }
        })
    }

    pub fn name(&self) -> &str {
        match self {
            Tree::Directory { name, .. } | Tree::File { name, .. } => name,
        }
    }

    pub fn chunk_id(&self) -> &ChunkIdentifier {
        match self {
            Tree::File { chunk_id, .. } => chunk_id,
            Tree::Directory { .. } => panic!("cannot get info from a directory"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Tree::Directory { children, .. } => children.is_empty(),
            Tree::File { .. } => false,
        }
    }

    /// Get the id of the tree.
    ///
    /// The id depends on the contents of the tree and will change if the node
    /// itself or one of its children changes.
    pub fn id(&self) -> String {
        let mut hasher = Sha256::new();

        // Separators are important as they prevent naming attacks and
        // disambiguate.
        //
        // A good rule of thumb for a hashing function, is to make sure that
        // before applying the hashing process, the input string should be 100%
        // decodable to its original form, without any ambiguity.
        match &self {
            Self::File { name, chunk_id } => {
                hasher.update(b"F(");
                hasher.update(name.as_bytes());
                hasher.update(b",");
                hasher.update(chunk_id.as_vec());
                hasher.update(b")");
            }
            &Self::Directory { name, ref children } => {
                hasher.update(b"D(");
                hasher.update(name.as_bytes());

                for child in children.values() {
                    hasher.update(b",");
                    hasher.update(child.id().as_bytes());
                }

                hasher.update(b")");
            }
        }

        format!("{:x}", hasher.finalize())
    }

    // Iterate over the tree, starting with its root and going doing the hierarchy.
    //
    // Directory nodes are guaranteed to be visited before the nodes they contain.
    pub fn iter(&self) -> TreeIterator<'_> {
        TreeIterator::new(self)
    }

    // Returns an iterator over the children of the tree.
    //
    // The iterator returns each `(CanonicalPath, Tree)` pair for all the files.
    // The returned `CanonicalPath` represents the complete path to the file.
    pub fn files(&self) -> TreeFilesIterator<'_> {
        TreeFilesIterator::new(self)
    }

    pub(crate) fn children_mut(&mut self) -> &mut BTreeMap<String, Self> {
        match self {
            Self::Directory { children, .. } => children,
            Self::File { .. } => unreachable!(),
        }
    }

    /// Find a node by its canonical path.
    pub fn find(&self, canonical_path: &CanonicalPath) -> Result<Option<&Self>> {
        if let Some((name, child_path)) = canonical_path.split_left() {
            let parent_name = self.name().to_string();

            if let Some(child) = self
                .get_direct_child_by_name(name)
                .with_parent_name(&parent_name)?
            {
                if let Some(child_path) = child_path {
                    child.find(&child_path).with_parent_name(&parent_name)
                } else {
                    // Final lookup: if there is a child, it's the one we got.
                    Ok(Some(child))
                }
            } else {
                // The children was not found: end of the search.
                Ok(None)
            }
        } else {
            // We were passed a root path: returns the tree itself.
            Ok(Some(self))
        }
    }

    /// Find a node by its canonical path.
    pub fn find_mut(&mut self, canonical_path: &CanonicalPath) -> Result<Option<&mut Self>> {
        if let Some((name, child_path)) = canonical_path.split_left() {
            let parent_name = self.name().to_string();

            if let Some(child) = self
                .get_direct_child_by_name_mut(name)
                .with_parent_name(&parent_name)?
            {
                if let Some(child_path) = child_path {
                    child.find_mut(&child_path).with_parent_name(&parent_name)
                } else {
                    // Final lookup: if there is a child, it's the one we got.
                    Ok(Some(child))
                }
            } else {
                // The children was not found: end of the search.
                Ok(None)
            }
        } else {
            // We were passed a root path: returns the tree itself.
            Ok(Some(self))
        }
    }

    /// Find a node by its canonical path.
    ///
    /// If the node doesn't exist, it will be created as well as its parents.
    ///
    /// If an intermediate node or the node itself isn't a directory, it becomes
    /// one and all file information is lost for the transformed nodes.
    ///
    /// Any newly created node is a directory node.
    pub fn find_or_replace(&mut self, canonical_path: &CanonicalPath) -> &mut Self {
        if let Some((name, child_path)) = canonical_path.split_left() {
            let child = self.get_or_replace_direct_child(name);

            if let Some(child_path) = child_path {
                child.find_or_replace(&child_path)
            } else {
                // Final lookup: if there is a child, it's the one we got.
                child
            }
        } else {
            // We were passed a root path: returns the tree itself.
            self
        }
    }

    /// Find a node by its canonical path.
    ///
    /// If the node doesn't exist, it will be created as well as its parents.
    ///
    /// If an intermediate node or the node itself isn't a directory, an error is returned.
    ///
    /// Any newly created node is a directory node.
    pub fn find_or_create(&mut self, canonical_path: &CanonicalPath) -> Result<&mut Self> {
        if let Some((name, child_path)) = canonical_path.split_left() {
            let parent_name = self.name().to_string();
            let child = self
                .get_or_create_direct_child(name)
                .with_parent_name(&parent_name)?;

            if let Some(child_path) = child_path {
                child
                    .find_or_create(&child_path)
                    .with_parent_name(&parent_name)
            } else {
                // Final lookup: if there is a child, it's the one we got.
                Ok(child)
            }
        } else {
            // We were passed a root path: returns the tree itself.
            Ok(self)
        }
    }

    /// Set a node by its canonical path.
    ///
    /// The node will be inserted or updated, with any
    /// intermediate directories created.
    ///
    /// If a node already exists at the given path, it will be replaced.
    ///
    /// # Returns
    ///
    /// The node that was removed or replaced, if any.
    pub fn set(&mut self, canonical_path: &CanonicalPath, tree: Self) -> Option<Self> {
        self.find_or_replace(canonical_path).set_direct_child(tree)
    }

    /// Add a node by its canonical path.
    ///
    /// The node will be inserted
    pub fn add(&mut self, canonical_path: &CanonicalPath, tree: Self) -> Result<&mut Self> {
        self.find_or_create(canonical_path)?
            .add_direct_child(tree)
            .with_parent_path(canonical_path)
    }

    /// Remove a node by its canonical path.
    ///
    /// If a node is removed, it is returned.
    ///
    /// If remove is called on the root path, the tree is emptied.
    pub fn remove(&mut self, canonical_path: &CanonicalPath) -> Option<Self> {
        if let Some((name, child_path)) = canonical_path.split_left() {
            if let Ok(Some(child)) = self.get_direct_child_by_name_mut(name) {
                if let Some(child_path) = child_path {
                    let result = child.remove(&child_path);

                    // If we removed a node and the child is now empty, we need to remove it as well.
                    if result.is_some() && child.is_empty() {
                        self.children_mut().remove(name);
                    }

                    result
                } else {
                    self.remove_direct_child_by_name(name)
                }
            } else {
                // The children was not found: end of the search.
                None
            }
        } else {
            // We were passed a root path: empty the tree and return the old
            // tree.
            Some(std::mem::replace(self, Self::empty()))
        }
    }

    /// Remove a file node by its canonical path if its info matches.
    ///
    /// If a node is removed, it is returned.
    ///
    /// If no such node is found, or the node is not a file, or the info doesn't match an error is returned.
    pub fn remove_file(
        &mut self,
        canonical_path: &CanonicalPath,
        chunk_id: &ChunkIdentifier,
    ) -> Result<Self> {
        if let Some((name, child_path)) = canonical_path.split_left() {
            let parent_name = self.name().to_string();
            if let Some(child) = self
                .get_direct_child_by_name_mut(name)
                .with_parent_name(&parent_name)?
            {
                if let Some(child_path) = child_path {
                    let result = child
                        .remove_file(&child_path, chunk_id)
                        .with_parent_name(&parent_name)?;

                    // If we removed a node and the child is now empty, we need to remove it as well.
                    if child.is_empty() {
                        self.children_mut().remove(name);
                    }

                    Ok(result)
                } else {
                    self.remove_direct_file_by_name(name, chunk_id)
                        .with_parent_name(&parent_name)
                }
            } else {
                // The children was not found: end of the search.
                Err(Error::file_does_not_exist(
                    canonical_path.clone().prepend(self.name()),
                ))
            }
        } else {
            // We were passed a root path: empty the tree and return the old
            // tree.
            if let Self::File { chunk_id: i, .. } = self {
                if i == chunk_id {
                    Ok(std::mem::replace(self, Self::empty()))
                } else {
                    Err(Error::file_content_mismatch(
                        canonical_path.clone(),
                        i.clone(),
                        chunk_id.clone(),
                    ))
                }
            } else {
                Err(Error::path_is_not_a_file(canonical_path.clone()))
            }
        }
    }

    // Update a file at the specified location if its hash matches the specified one.
    pub fn update_file(
        &mut self,
        canonical_path: &CanonicalPath,
        old_chunk_id: &ChunkIdentifier,
        new_chunk_id: &ChunkIdentifier,
    ) -> Result<&mut Self> {
        match self.find_mut(canonical_path)? {
            Some(child) => {
                if let Self::File { chunk_id: i, .. } = child {
                    if i == old_chunk_id {
                        *i = new_chunk_id.clone();
                        Ok(child)
                    } else {
                        Err(Error::file_content_mismatch(
                            canonical_path.clone(),
                            i.clone(),
                            new_chunk_id.clone(),
                        ))
                    }
                } else {
                    Err(Error::path_is_not_a_file(canonical_path.clone()))
                }
            }
            None => Err(Error::file_does_not_exist(canonical_path.clone())),
        }
    }

    /// Returns a direct child with the specified name if one exists.
    pub fn get_direct_child_by_name(&self, name: &str) -> Result<Option<&Self>> {
        match self {
            Self::File { .. } => Err(Error::path_is_not_a_directory(CanonicalPath::root())),
            Self::Directory { children, .. } => Ok(children.get(name)),
        }
    }

    /// Returns a direct child with the specified name if one exists.
    pub fn get_direct_child_by_name_mut(&mut self, name: &str) -> Result<Option<&mut Self>> {
        match self {
            Self::File { .. } => Err(Error::path_is_not_a_directory(CanonicalPath::root())),
            Self::Directory { children, .. } => Ok(children.get_mut(name)),
        }
    }

    /// Returns a direct child with the specified name if one exists.
    ///
    /// If the current node is not a directory node, it becomes one and all file
    /// information is lost.
    fn get_or_replace_direct_child(&mut self, name: &str) -> &mut Self {
        match self {
            Self::File { .. } => {
                *self = Self::directory(
                    self.name().to_string(),
                    [Self::directory(name.to_string(), [])],
                );
                self.children_mut().get_mut(name).unwrap()
            }
            Self::Directory { children, .. } => children
                .entry(name.to_string())
                .or_insert_with(|| Self::directory(name.to_string(), [])),
        }
    }

    /// Returns a direct child with the specified name if one exists.
    ///
    /// If the current node is not a directory node, None is returned.
    fn get_or_create_direct_child(&mut self, name: &str) -> Result<&mut Self> {
        match self {
            Self::File { .. } => Err(Error::path_is_not_a_directory(CanonicalPath::root())),
            Self::Directory { children, .. } => Ok(children
                .entry(name.to_string())
                .or_insert_with(|| Self::directory(name.to_string(), []))),
        }
    }

    /// Set a direct child with the specified tree and returns the old one if
    /// any.
    ///
    /// If the node is not a directory node, it becomes one and all file
    /// information is lost.
    fn set_direct_child(&mut self, tree: Self) -> Option<Self> {
        match self {
            Self::File { .. } => {
                *self = Self::directory(self.name().to_string(), [tree]);

                None
            }
            Self::Directory { children, .. } => children.insert(tree.name().to_string(), tree),
        }
    }

    /// Set a direct child with the specified tree and returns a reference to
    /// it.
    ///
    /// If the node is not a directory node, it becomes one and all file
    /// information is lost.
    fn add_direct_child(&mut self, tree: Self) -> Result<&mut Self> {
        match self {
            Self::File { .. } => Err(Error::path_is_not_a_directory(CanonicalPath::root())),
            Self::Directory { children, .. } => match children.entry(tree.name().to_string()) {
                Entry::Occupied(entry) => {
                    let old = entry.get();
                    if old != &tree {
                        // If the previous child is an empty directy, allow its replacement.
                        if old.is_empty() {
                            let value = entry.into_mut();

                            *value = tree;

                            Ok(value)
                        } else {
                            Err(Error::file_already_exists(CanonicalPath::new_from_name(
                                old.name(),
                            )))
                        }
                    } else {
                        Ok(entry.into_mut())
                    }
                }
                Entry::Vacant(entry) => Ok(entry.insert(tree)),
            },
        }
    }

    /// Remove a direct child of the tree by its name.
    fn remove_direct_child_by_name(&mut self, name: &str) -> Option<Self> {
        match self {
            Self::File { .. } => None,
            Self::Directory { children, .. } => children.remove(name),
        }
    }

    /// Remove a direct file child of the tree by its name.
    fn remove_direct_file_by_name(
        &mut self,
        name: &str,
        chunk_id: &ChunkIdentifier,
    ) -> Result<Self> {
        match self {
            Self::File { .. } => Err(Error::path_is_not_a_directory(CanonicalPath::root())),
            Self::Directory { children, .. } => match children.entry(name.to_string()) {
                Entry::Occupied(entry) => match entry.get() {
                    Self::File {
                        chunk_id: entry_info,
                        ..
                    } => {
                        if entry_info == chunk_id {
                            Ok(entry.remove())
                        } else {
                            Err(Error::file_content_mismatch(
                                CanonicalPath::new_from_name(name),
                                chunk_id.clone(),
                                entry_info.clone(),
                            ))
                        }
                    }
                    Self::Directory { .. } => Err(Error::path_is_not_a_file(
                        CanonicalPath::new_from_name(name),
                    )),
                },
                Entry::Vacant(_) => Err(Error::file_does_not_exist(CanonicalPath::new_from_name(
                    name,
                ))),
            },
        }
    }

    pub fn with_changes<'c>(
        mut self,
        changes: impl IntoIterator<Item = &'c Change> + Clone,
    ) -> Result<Self> {
        // We need to iterate first over the deletions, because it could make some additions fail.
        for change in changes.clone() {
            if let ChangeType::Delete {
                old_chunk_id: old_info,
            } = change.change_type()
            {
                self.remove_file(change.canonical_path(), old_info)
                    .map_other_err("failed to apply file deletion")?;
            }
        }

        // Deletions were done, now we can add and edit files.
        for change in changes {
            match change.change_type() {
                ChangeType::Add {
                    new_chunk_id: new_hash,
                } => {
                    let (parent_path, name) = change.canonical_path().split();
                    let name = if let Some(name) = name {
                        name
                    } else {
                        continue;
                    };

                    // If the addition is conflicting, an error will be raised.
                    self.add(&parent_path, Self::file(name.to_string(), new_hash.clone()))
                        .map_other_err("failed to apply file addition")?;
                }
                ChangeType::Edit {
                    old_chunk_id: old_info,
                    new_chunk_id: new_info,
                } => {
                    self.update_file(change.canonical_path(), old_info, new_info)
                        .map_other_err("failed to apply file edit")?;
                }
                ChangeType::Delete { .. } => {}
            }
        }

        Ok(self)
    }

    /// Returns the list of changes to apply to an empty tree, to get to this.
    pub fn as_forward_changes(&self) -> BTreeSet<Change> {
        let mut changes = BTreeSet::new();

        match self {
            Self::File {
                name,
                chunk_id: info,
            } => {
                changes.insert(Change::new(
                    CanonicalPath::new_from_name(name),
                    ChangeType::Add {
                        new_chunk_id: info.clone(),
                    },
                ));
            }
            Self::Directory {
                name: parent_name,
                children,
            } => {
                changes.extend(
                    children
                        .iter()
                        .flat_map(|(_, child)| child.as_forward_changes())
                        .map(|change| change.with_parent_name(parent_name)),
                );
            }
        };

        changes
    }

    /// Returns the list of changes to apply to this tree, to get to an empty one.
    pub fn as_backward_changes(&self) -> BTreeSet<Change> {
        self.as_forward_changes()
            .into_iter()
            .map(Change::into_invert)
            .collect()
    }

    pub fn get_changes_to(&self, tree: &Self) -> BTreeSet<Change> {
        let mut changes = BTreeSet::new();

        match self {
            Self::File {
                name,
                chunk_id: info,
            } => match tree {
                Self::File {
                    name: tree_name,
                    chunk_id: tree_info,
                } => {
                    if name == tree_name {
                        if info == tree_info {
                            return changes;
                        }

                        changes.insert(Change::new(
                            CanonicalPath::new_from_name(name),
                            ChangeType::Edit {
                                old_chunk_id: info.clone(),
                                new_chunk_id: tree_info.clone(),
                            },
                        ));
                    } else {
                        changes.insert(Change::new(
                            CanonicalPath::new_from_name(name),
                            ChangeType::Delete {
                                old_chunk_id: info.clone(),
                            },
                        ));
                        changes.insert(Change::new(
                            CanonicalPath::new_from_name(tree_name),
                            ChangeType::Add {
                                new_chunk_id: tree_info.clone(),
                            },
                        ));
                    }
                }
                Self::Directory { .. } => {
                    changes.insert(Change::new(
                        CanonicalPath::new_from_name(name),
                        ChangeType::Delete {
                            old_chunk_id: info.clone(),
                        },
                    ));
                    changes.extend(tree.as_forward_changes());
                }
            },
            Self::Directory { name, children } => match tree {
                Self::File {
                    name: tree_name,
                    chunk_id: tree_info,
                } => {
                    changes.extend(self.as_backward_changes());
                    changes.insert(Change::new(
                        CanonicalPath::new_from_name(tree_name),
                        ChangeType::Add {
                            new_chunk_id: tree_info.clone(),
                        },
                    ));
                }
                Self::Directory {
                    name: tree_name,
                    children: tree_children,
                } => {
                    if name != tree_name {
                        changes.extend(self.as_backward_changes());
                        changes.extend(tree.as_forward_changes());
                    } else {
                        // First we add deletions for all the children in the current tree that do not exist in the new tree.
                        for (child_name, child) in children {
                            if !tree_children.contains_key(child_name) {
                                changes.extend(
                                    child
                                        .as_backward_changes()
                                        .into_iter()
                                        .map(|change| change.with_parent_name(name)),
                                );
                            }
                        }

                        // Then we register changes and additions.
                        for (tree_child_name, tree_child) in tree_children {
                            if let Some(child) = children.get(tree_child_name) {
                                changes.extend(
                                    child
                                        .get_changes_to(tree_child)
                                        .into_iter()
                                        .map(|change| change.with_parent_name(name)),
                                );
                            } else {
                                changes.extend(
                                    tree_child
                                        .as_forward_changes()
                                        .into_iter()
                                        .map(|change| change.with_parent_name(name)),
                                );
                            }
                        }
                    }
                }
            },
        }

        changes
    }
}

impl<'t> IntoIterator for &'t Tree {
    type Item = (CanonicalPath, Self);
    type IntoIter = TreeIterator<'t>;

    fn into_iter(self) -> TreeIterator<'t> {
        TreeIterator::new(self)
    }
}

pub struct TreeIterator<'t> {
    node: Option<&'t Tree>,
    stack: Vec<std::collections::btree_map::Values<'t, String, Tree>>,
    path: CanonicalPath,
}

impl<'t> TreeIterator<'t> {
    pub fn new(tree: &'t Tree) -> Self {
        Self {
            node: Some(tree),
            stack: Vec::new(),
            path: CanonicalPath::root(),
        }
    }
}

impl<'t> Iterator for TreeIterator<'t> {
    type Item = (CanonicalPath, &'t Tree);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(node) = self.node.take() {
                break match node {
                    Tree::File { name, .. } => Some((self.path.clone().append(name), node)),
                    Tree::Directory { name, children } => {
                        self.stack.push(children.values());
                        self.path = self.path.clone().append(name);

                        Some((self.path.clone(), node))
                    }
                };
            }

            let next_tree = match self.stack.last_mut() {
                Some(child_iterator) => child_iterator.next(),
                None => break None,
            };

            if let Some(tree) = next_tree {
                self.node = Some(tree);
            } else {
                // We reached the end of the last children iterator: pop the
                // stack and continue iterating on the parent one.
                self.stack.pop();
                self.path.pop();
            }
        }
    }
}

pub struct TreeFilesIterator<'t> {
    file_node: Option<&'t Tree>,
    stack: Vec<std::collections::btree_map::Values<'t, String, Tree>>,
    path: CanonicalPath,
}

impl<'t> TreeFilesIterator<'t> {
    pub fn new(tree: &'t Tree) -> Self {
        match tree {
            Tree::File { .. } => Self {
                file_node: Some(tree),
                stack: Vec::new(),
                path: CanonicalPath::root(),
            },
            Tree::Directory { name, children } => Self {
                file_node: None,
                stack: vec![children.values()],
                path: CanonicalPath::root().append(name),
            },
        }
    }
}

impl<'t> Iterator for TreeFilesIterator<'t> {
    type Item = (CanonicalPath, &'t Tree);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(file_node) = self.file_node.take() {
                break Some((self.path.clone().append(file_node.name()), file_node));
            }

            let next_tree = match self.stack.last_mut() {
                Some(child_iterator) => child_iterator.next(),
                None => break None,
            };

            match match next_tree {
                Some(tree) => match tree {
                    Tree::File { .. } => {
                        self.file_node = Some(tree);
                        continue;
                    }
                    Tree::Directory { name, children } => {
                        // Triggers a push of the stack below.
                        Some((name, children))
                    }
                },
                None => None, // Triggers a pop of the stack below.
            } {
                None => {
                    // We reached the end of the last children iterator: pop the
                    // stack and continue iterating on the parent one.
                    self.stack.pop();
                    self.path.pop();
                }
                Some((name, children)) => {
                    // We reached a new children iterator: push it and continue.
                    self.stack.push(children.values());
                    self.path = self.path.clone().append(name);
                }
            }
        }
    }
}

/// A filter that can be used to filter out files and directories from a tree.
#[derive(Debug, Clone, Default)]
pub struct TreeFilter {
    pub inclusion_rules: BTreeSet<CanonicalPath>,
    pub exclusion_rules: BTreeSet<CanonicalPath>,
}

impl TreeFilter {
    /// Checks whether a specified path should be excluded, according the defined
    /// exclusion rules.
    ///
    /// A path is excluded if it matches any of the exclusion rules.
    pub fn check_exclusion(&self, canonical_path: &CanonicalPath) -> Result<()> {
        for exclusion_rule in &self.exclusion_rules {
            if exclusion_rule.contains(canonical_path) {
                return Err(Error::path_excluded(
                    canonical_path.clone(),
                    exclusion_rule.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Checks whether a specified path should be included, according the defined
    /// inclusion rules.
    ///
    /// A path is included if it intersects any of the inclusion rules.
    pub fn check_inclusion(&self, canonical_path: &CanonicalPath) -> Result<()> {
        if self.inclusion_rules.is_empty() {
            return Ok(());
        }

        for inclusion_rule in &self.inclusion_rules {
            if canonical_path.intersects(inclusion_rule) {
                return Ok(());
            }
        }

        Err(Error::path_not_included(canonical_path.clone()))
    }

    /// Checks whether a specified path should be included or excluded, according the defined
    /// inclusion and exclusion rules.
    ///
    /// A path is included if it matches any of the inclusion rules but none of the exclusion rules.
    pub fn check(&self, canonical_path: &CanonicalPath) -> Result<()> {
        self.check_inclusion(canonical_path)?;
        self.check_exclusion(canonical_path)
    }
}

#[cfg(test)]
mod tests {
    use lgn_content_store2::{MemoryProvider, SmallContentProvider};

    use super::*;

    fn d(name: &str, children: impl IntoIterator<Item = Tree>) -> Tree {
        Tree::directory(name.to_string(), children)
    }

    fn f(csp: &Chunker<impl ContentProvider + Send + Sync>, name: &str, data: &str) -> Tree {
        Tree::file(name.to_string(), id(csp, data))
    }

    fn id(csp: &Chunker<impl ContentProvider + Send + Sync>, data: &str) -> ChunkIdentifier {
        csp.get_chunk_identifier(data.as_bytes())
    }

    fn cp(s: &str) -> CanonicalPath {
        CanonicalPath::new(s).unwrap()
    }

    fn add(csp: &Chunker<impl ContentProvider + Send + Sync>, s: &str, new_data: &str) -> Change {
        Change::new(
            cp(s),
            ChangeType::Add {
                new_chunk_id: id(csp, new_data),
            },
        )
    }

    fn edit(
        csp: &Chunker<impl ContentProvider + Send + Sync>,
        s: &str,
        old_data: &str,
        new_data: &str,
    ) -> Change {
        Change::new(
            cp(s),
            ChangeType::Edit {
                old_chunk_id: id(csp, old_data),
                new_chunk_id: id(csp, new_data),
            },
        )
    }

    fn delete(
        csp: &Chunker<impl ContentProvider + Send + Sync>,
        s: &str,
        old_data: &str,
    ) -> Change {
        Change::new(
            cp(s),
            ChangeType::Delete {
                old_chunk_id: id(csp, old_data),
            },
        )
    }

    #[tokio::test]
    async fn test_tree_from_root() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));
        let root = &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/canonical_path");

        // Let's resolve the root.
        let tree = Tree::from_root(csp, root, &TreeFilter::default())
            .await
            .unwrap();

        fn f(csp: &Chunker<impl ContentProvider + Send + Sync>, name: &str, data: &str) -> Tree {
            Tree::file(name.to_string(), id(csp, data))
        }

        assert_eq!(
            tree,
            d(
                "",
                [
                    d(
                        "fruits",
                        [
                            f(csp, "apple.txt", "I am an apple."),
                            f(csp, "orange.txt", "I am an orange."),
                            f(csp, "tomato.txt", "I am a tomato."),
                        ]
                    ),
                    d("vegetables", [f(csp, "carrot.txt", "I am a carot.")]),
                ],
            )
        );

        let tree_filter = TreeFilter {
            inclusion_rules: [cp("/vegetables/carrot.txt")].into(),
            exclusion_rules: BTreeSet::new(),
        };

        let tree = Tree::from_root(csp, root, &tree_filter).await.unwrap();

        assert_eq!(
            tree,
            d(
                "",
                [d("vegetables", [f(csp, "carrot.txt", "I am a carot.")])],
            )
        );
    }

    #[test]
    fn test_tree_find() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));

        let mut tree = d(
            "",
            [
                d("a", [f(csp, "e", "he"), d("f", [f(csp, "g", "hg")])]),
                d("b", []),
                f(csp, "c", "hc"),
                f(csp, "d", "hd"),
            ],
        );

        assert_eq!(tree.find(&cp("/a/f/g")).unwrap(), Some(&f(csp, "g", "hg")));
        assert_eq!(
            tree.find(&cp("/a/f")).unwrap(),
            Some(&d("f", [f(csp, "g", "hg")]))
        );
        assert_eq!(tree.find(&cp("/c")).unwrap(), Some(&f(csp, "c", "hc")));
        assert_eq!(tree.find(&cp("/x")).unwrap(), None);
        assert_eq!(tree.find(&cp("/")).unwrap(), Some(&tree));

        match tree.find(&cp("/c/x")) {
            Err(Error::PathIsNotADirectory { canonical_path }) => {
                assert_eq!(canonical_path, cp("/c"));
            }
            Err(err) => panic!("expected PathIsNotADirectory, got :`{:?}`", err),
            _ => panic!("expected PathIsNotADirectory"),
        }

        match tree.find(&cp("/a/f/g/x")) {
            Err(Error::PathIsNotADirectory { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/f/g"));
            }
            Err(err) => panic!("expected PathIsNotADirectory, got :`{:?}`", err),
            _ => panic!("expected PathIsNotADirectory"),
        }

        // Same tests, but with the mutable version of the function.

        assert_eq!(
            tree.find_mut(&cp("/a/f/g")).unwrap(),
            Some(&mut f(csp, "g", "hg"))
        );
        assert_eq!(
            tree.find_mut(&cp("/a/f")).unwrap(),
            Some(&mut d("f", [f(csp, "g", "hg")]))
        );
        assert_eq!(
            tree.find_mut(&cp("/c")).unwrap(),
            Some(&mut f(csp, "c", "hc"))
        );
        assert_eq!(tree.find_mut(&cp("/x")).unwrap(), None);

        let mut tree_copy = tree.clone();
        assert_eq!(tree.find_mut(&cp("/")).unwrap(), Some(&mut tree_copy));

        match tree.find_mut(&cp("/c/x")) {
            Err(Error::PathIsNotADirectory { canonical_path }) => {
                assert_eq!(canonical_path, cp("/c"));
            }
            Err(err) => panic!("expected PathIsNotADirectory, got :`{:?}`", err),
            _ => panic!("expected PathIsNotADirectory"),
        }

        match tree.find_mut(&cp("/a/f/g/x")) {
            Err(Error::PathIsNotADirectory { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/f/g"));
            }
            Err(err) => panic!("expected PathIsNotADirectory, got :`{:?}`", err),
            _ => panic!("expected PathIsNotADirectory"),
        }
    }

    #[test]
    fn test_tree_sort() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));

        let tree = d(
            "",
            [
                d("a", [f(csp, "e", "he"), d("f", [f(csp, "g", "hg")])]),
                f(csp, "d", "hd"),
                f(csp, "c", "hc"),
                d("b", []),
            ],
        );

        assert_eq!(
            tree,
            d(
                "",
                [
                    d("a", [d("f", [f(csp, "g", "hg")]), f(csp, "e", "he")]),
                    d("b", []),
                    f(csp, "c", "hc"),
                    f(csp, "d", "hd"),
                ]
            )
        );
    }

    #[test]
    fn test_tree_manipulation() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));

        let mut tree = d(
            "",
            [
                d("a", [d("f", [f(csp, "g", "hg")]), f(csp, "e", "he")]),
                f(csp, "c", "hc"),
                f(csp, "d", "hd"),
            ],
        );

        assert_eq!(tree.remove(&cp("/a/f/g")), Some(f(csp, "g", "hg")));

        assert_eq!(
            tree,
            d(
                "",
                [
                    d("a", [f(csp, "e", "he")]),
                    f(csp, "c", "hc"),
                    f(csp, "d", "hd"),
                ]
            )
        );

        assert_eq!(tree.remove(&cp("/a")), Some(d("a", [f(csp, "e", "he")])));

        assert_eq!(tree, d("", [f(csp, "c", "hc"), f(csp, "d", "hd"),]));

        assert_eq!(tree.remove(&cp("/x")), None);

        assert_eq!(tree.set(&cp("/a/b/c/d"), f(csp, "z", "hz")), None);

        assert_eq!(
            tree,
            d(
                "",
                [
                    d("a", [d("b", [d("c", [d("d", [f(csp, "z", "hz")])])])]),
                    f(csp, "c", "hc"),
                    f(csp, "d", "hd"),
                ]
            )
        );

        // Adding a file that already exists is fine if the file is identical.
        assert_eq!(
            *tree.add(&cp("/a/b/c/d"), f(csp, "z", "hz")).unwrap(),
            f(csp, "z", "hz")
        );

        // Cannot add a file that already exists with a different content.
        match tree.add(&cp("/a/b/c/d"), f(csp, "z", "hz2")) {
            Err(Error::FileAlreadyExists { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/b/c/d/z"));
            }
            Err(err) => panic!("expected FileAlreadyExists, got: {}", err),
            Ok(_) => panic!("expected FileAlreadyExists"),
        }

        // Cannot add a file to a non-directory, direct.
        match tree.add(&cp("/a/b/c/d/z"), f(csp, "x", "hx")) {
            Err(Error::PathIsNotADirectory { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/b/c/d/z"));
            }
            _ => panic!("expected PathIsNotADirectory"),
        }

        // Cannot add a file to a non-directory, indirect.
        match tree.add(&cp("/a/b/c/d/z/z/z/z"), f(csp, "x", "hx")) {
            Err(Error::PathIsNotADirectory { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/b/c/d/z"));
            }
            _ => panic!("expected PathIsNotADirectory"),
        }

        assert_eq!(
            *tree.add(&cp("/a/b/c/d"), f(csp, "x", "hx")).unwrap(),
            f(csp, "x", "hx")
        );

        assert_eq!(
            tree,
            d(
                "",
                [
                    d(
                        "a",
                        [d(
                            "b",
                            [d("c", [d("d", [f(csp, "z", "hz"), f(csp, "x", "hx")])])]
                        )]
                    ),
                    f(csp, "c", "hc"),
                    f(csp, "d", "hd"),
                ]
            )
        );

        assert_eq!(
            tree.remove_file(&cp("/a/b/c/d/x"), &id(csp, "hx")).unwrap(),
            f(csp, "x", "hx")
        );

        // File does not exist anymore.
        match tree.remove_file(&cp("/a/b/c/d/x"), &id(csp, "hx")) {
            Err(Error::FileDoesNotExist { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/b/c/d/x"));
            }
            _ => panic!("expected FileDoesNotExist"),
        }

        // Intermediate path does not exist.
        match tree.remove_file(&cp("/a/a/a/a/x"), &id(csp, "hx")) {
            Err(Error::FileDoesNotExist { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/a/a/a/x"));
            }
            _ => panic!("expected FileDoesNotExist"),
        }

        // File exists but with a different content.
        match tree.remove_file(&cp("/a/b/c/d/z"), &id(csp, "hz2")) {
            Err(Error::FileContentMistmatch {
                canonical_path,
                expected_chunk_id,
                chunk_id,
            }) => {
                assert_eq!(canonical_path, cp("/a/b/c/d/z"));
                assert_eq!(expected_chunk_id, id(csp, "hz2"));
                assert_eq!(chunk_id, id(csp, "hz"));
            }
            _ => panic!("expected FileContentMistmatch"),
        }

        // Trying to remove a file on a file.
        match tree.remove_file(&cp("/a/b/c/d/z/z"), &id(csp, "hz")) {
            Err(Error::PathIsNotADirectory { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/b/c/d/z"));
            }
            Err(err) => panic!("expected PathIsNotADirectory, got: {:?}", err),
            _ => panic!("expected PathIsNotADirectory"),
        }

        assert_eq!(
            tree,
            d(
                "",
                [
                    d("a", [d("b", [d("c", [d("d", [f(csp, "z", "hz")])])])]),
                    f(csp, "c", "hc"),
                    f(csp, "d", "hd"),
                ]
            )
        );

        // No-op update should be fine.
        assert_eq!(
            tree.update_file(&cp("/a/b/c/d/z"), &id(csp, "hz"), &id(csp, "hz"))
                .unwrap(),
            &mut f(csp, "z", "hz")
        );

        assert_eq!(
            tree.update_file(&cp("/a/b/c/d/z"), &id(csp, "hz"), &id(csp, "hz2"))
                .unwrap(),
            &mut f(csp, "z", "hz2")
        );

        // File does not exist anymore.
        match tree.update_file(&cp("/a/b/c/d/x"), &id(csp, "hx"), &id(csp, "hx2")) {
            Err(Error::FileDoesNotExist { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/b/c/d/x"));
            }
            _ => panic!("expected FileDoesNotExist"),
        }

        // Intermediate path does not exist.
        match tree.update_file(&cp("/a/a/a/a/x"), &id(csp, "hx"), &id(csp, "hx2")) {
            Err(Error::FileDoesNotExist { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/a/a/a/x"));
            }
            _ => panic!("expected FileDoesNotExist"),
        }

        // Trying to remove a file on a file.
        match tree.update_file(&cp("/a/b/c/d"), &id(csp, "hz"), &id(csp, "hz2")) {
            Err(Error::PathIsNotAFile { canonical_path }) => {
                assert_eq!(canonical_path, cp("/a/b/c/d"));
            }
            Err(err) => panic!("expected PathIsNotAFile, got: {:?}", err),
            _ => panic!("expected PathIsNotAFile"),
        }
    }

    #[test]
    fn test_tree_iter() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));

        let tree = d(
            "",
            [
                d("a", [f(csp, "e", "he"), d("f", [f(csp, "g", "hg")])]),
                f(csp, "d", "hd"),
                f(csp, "c", "hc"),
                d("b", []),
            ],
        );

        assert_eq!(
            tree.iter().collect::<Vec<(CanonicalPath, &Tree)>>(),
            [
                (cp("/"), &tree),
                (
                    cp("/a"),
                    &d("a", [f(csp, "e", "he"), d("f", [f(csp, "g", "hg")])])
                ),
                (cp("/a/e"), &f(csp, "e", "he")),
                (cp("/a/f"), &d("f", [f(csp, "g", "hg")])),
                (cp("/a/f/g"), &f(csp, "g", "hg")),
                (cp("/b"), &d("b", [])),
                (cp("/c"), &f(csp, "c", "hc")),
                (cp("/d"), &f(csp, "d", "hd")),
            ]
        );

        let tree = f(csp, "foo", "hfoo");

        assert_eq!(
            tree.files().collect::<Vec<(CanonicalPath, &Tree)>>(),
            [(cp("/foo"), &tree)]
        );
    }

    #[test]
    fn test_tree_files() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));

        let tree = d(
            "",
            [
                d("a", [f(csp, "e", "he"), d("f", [f(csp, "g", "hg")])]),
                f(csp, "d", "hd"),
                f(csp, "c", "hc"),
                d("b", []),
            ],
        );

        assert_eq!(
            tree.files().collect::<Vec<(CanonicalPath, &Tree)>>(),
            [
                (cp("/a/e"), &f(csp, "e", "he")),
                (cp("/a/f/g"), &f(csp, "g", "hg")),
                (cp("/c"), &f(csp, "c", "hc")),
                (cp("/d"), &f(csp, "d", "hd")),
            ]
        );

        let tree = f(csp, "foo", "hfoo");

        assert_eq!(
            tree.files().collect::<Vec<(CanonicalPath, &Tree)>>(),
            [(cp("/foo"), &f(csp, "foo", "hfoo")),]
        );
    }

    #[test]
    fn test_tree_filter() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));
        let tree_filter = TreeFilter {
            inclusion_rules: [cp("/a/b")].into(),
            exclusion_rules: [cp("/a/b/c")].into(),
        };

        assert!(tree_filter.check_exclusion(&cp("/")).is_ok());
        assert!(tree_filter.check_exclusion(&cp("/a")).is_ok());
        assert!(tree_filter.check_exclusion(&cp("/a/b")).is_ok());
        assert!(tree_filter.check_exclusion(&cp("/a/b/c")).is_err());
        assert!(tree_filter.check_exclusion(&cp("/a/b/c/d")).is_err());
        assert!(tree_filter.check_exclusion(&cp("/a/b/d")).is_ok());
        assert!(tree_filter.check_exclusion(&cp("/a/c")).is_ok());

        assert!(tree_filter.check_inclusion(&cp("/")).is_ok());
        assert!(tree_filter.check_inclusion(&cp("/a")).is_ok());
        assert!(tree_filter.check_inclusion(&cp("/a/b")).is_ok());
        assert!(tree_filter.check_inclusion(&cp("/a/b/c")).is_ok());
        assert!(tree_filter.check_inclusion(&cp("/a/b/c/d")).is_ok());
        assert!(tree_filter.check_inclusion(&cp("/a/b/d")).is_ok());
        assert!(tree_filter.check_inclusion(&cp("/a/c")).is_err());

        assert!(tree_filter.check(&cp("/")).is_ok());
        assert!(tree_filter.check(&cp("/a")).is_ok());
        assert!(tree_filter.check(&cp("/a/b")).is_ok());
        assert!(tree_filter.check(&cp("/a/b/c")).is_err());
        assert!(tree_filter.check(&cp("/a/b/c/d")).is_err());
        assert!(tree_filter.check(&cp("/a/b/d")).is_ok());
        assert!(tree_filter.check(&cp("/a/c")).is_err());

        let tree = d(
            "",
            [d(
                "a",
                [
                    d("b", [d("c", [f(csp, "d", "hd")]), f(csp, "d", "hd")]),
                    f(csp, "c", "hc"),
                ],
            )],
        );

        assert_eq!(
            tree.filter(&tree_filter).unwrap(),
            d("", [d("a", [d("b", [f(csp, "d", "hd")])],)])
        );
    }

    #[test]
    fn test_tree_with_changes() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));

        let tree = d(
            "",
            [
                d("a", [f(csp, "e", "he"), d("f", [f(csp, "g", "hg")])]),
                d("b", []),
                f(csp, "c", "hc"),
                f(csp, "d", "hd"),
                d("e", []),
                d("f", [d("g", [f(csp, "h", "hh")])]),
                d("h", [f(csp, "i", "hi")]),
            ],
        );

        let tree_with_changes = tree
            .with_changes(&[
                add(csp, "/a/f/h", "hh"),
                add(csp, "/b", "hb"),
                edit(csp, "/a/e", "he", "he2"),
                delete(csp, "/c", "hc"),
                delete(csp, "/f/g/h", "hh"),
                add(csp, "/f/g", "gh"),
                delete(csp, "/h/i", "hi"),
            ])
            .unwrap();

        assert_eq!(
            tree_with_changes,
            d(
                "",
                [
                    d(
                        "a",
                        [
                            f(csp, "e", "he2"),
                            d("f", [f(csp, "g", "hg"), f(csp, "h", "hh")])
                        ]
                    ),
                    f(csp, "b", "hb"),
                    f(csp, "d", "hd"),
                    d("e", []),
                    d("f", [f(csp, "g", "gh")]),
                ]
            )
        );
    }

    #[test]
    fn test_tree_as_forward_changes() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));

        let tree = d(
            "",
            [
                d("a", [f(csp, "e", "he"), d("f", [f(csp, "g", "hg")])]),
                d("b", []),
                f(csp, "c", "hc"),
                f(csp, "d", "hd"),
                d("e", []),
                d("f", [d("g", [f(csp, "h", "hh")])]),
            ],
        );

        assert_eq!(
            tree.as_forward_changes(),
            [
                add(csp, "/a/e", "he"),
                add(csp, "/a/f/g", "hg"),
                add(csp, "/c", "hc"),
                add(csp, "/d", "hd"),
                add(csp, "/f/g/h", "hh"),
            ]
            .into()
        );
    }

    #[test]
    fn test_tree_as_backward_changes() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));
        let tree = d(
            "",
            [
                d("a", [f(csp, "e", "he"), d("f", [f(csp, "g", "hg")])]),
                d("b", []),
                f(csp, "c", "hc"),
                f(csp, "d", "hd"),
                d("e", []),
                d("f", [d("g", [f(csp, "h", "hh")])]),
            ],
        );

        assert_eq!(
            tree.as_backward_changes(),
            [
                delete(csp, "/a/e", "he"),
                delete(csp, "/a/f/g", "hg"),
                delete(csp, "/c", "hc"),
                delete(csp, "/d", "hd"),
                delete(csp, "/f/g/h", "hh"),
            ]
            .into()
        );
    }

    #[test]
    fn test_tree_changes_to() {
        let csp = &Chunker::new(SmallContentProvider::new(MemoryProvider::new()));
        let from_tree = d(
            "",
            [
                d("a", [f(csp, "e", "he"), d("f", [f(csp, "g", "hg")])]),
                d("b", []),
                f(csp, "c", "hc"),
                f(csp, "d", "hd"),
                d("e", []),
                d("f", [d("g", [f(csp, "h", "hh")])]),
            ],
        );
        let to_tree = d(
            "",
            [
                d("a", [f(csp, "z", "hz"), d("f", [f(csp, "g", "hg2")])]),
                d("b", []),
                d("c", []),
                f(csp, "d", "hd"),
                f(csp, "e", "he"),
                d("f", [d("g", [f(csp, "h", "hh")])]),
            ],
        );

        let changes = from_tree.get_changes_to(&to_tree);

        assert_eq!(
            changes,
            [
                delete(csp, "/a/e", "he"),
                add(csp, "/a/z", "hz"),
                edit(csp, "/a/f/g", "hg", "hg2"),
                delete(csp, "/c", "hc"),
                add(csp, "/e", "he"),
            ]
            .into()
        );

        // Reverse changes.
        let changes = to_tree.get_changes_to(&from_tree);

        assert_eq!(
            changes,
            [
                add(csp, "/a/e", "he"),
                delete(csp, "/a/z", "hz"),
                edit(csp, "/a/f/g", "hg2", "hg"),
                add(csp, "/c", "hc"),
                delete(csp, "/e", "he"),
            ]
            .into()
        );

        // Diffing with self should yield no changes.
        let changes = from_tree.get_changes_to(&from_tree);

        assert_eq!(changes, [].into(),);
    }
}
