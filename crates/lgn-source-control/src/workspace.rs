use std::{collections::BTreeSet, sync::Arc};

use lgn_content_store::{
    indexing::{
        self, BasicIndexer, IndexKey, ResourceByteReader, ResourceByteWriter, ResourceExists,
        ResourceIdentifier, ResourceReader, ResourceWriter, StringPathIndexer, Tree,
        TreeIdentifier, TreeLeafNode, TreeWriter,
    },
    Provider,
};
use lgn_tracing::error;
#[cfg(feature = "verbose")]
use lgn_tracing::info;

use crate::{
    Branch, Change, Commit, CommitId, Error, Index, ListBranchesQuery, ListCommitsQuery,
    RepositoryIndex, RepositoryName, Result,
};

/// Represents a workspace.
pub struct Workspace<MainIndexer> {
    index: Box<dyn Index>,
    transaction: Provider,
    branch_name: String,
    main_indexer: MainIndexer,
    path_indexer: StringPathIndexer,
    content_id: ContentId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentId {
    main_index_tree_id: TreeIdentifier,
    path_index_tree_id: TreeIdentifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Staging {
    StagedAndUnstaged,
    StagedOnly,
    UnstagedOnly,
}

pub enum CommitMode {
    /// In this mode committing staged files containing no changes or calling commit with no staged changes is treated as error.
    Strict,
    /// In this mode staged files with no changes will be ignored/skipped. Committing no changes is effectively a noop.
    Lenient,
}

impl Staging {
    pub fn from_bool(staged_only: bool, unstaged_only: bool) -> Self {
        assert!(
            !(staged_only && unstaged_only),
            "staged_only and unstaged_only cannot both be true"
        );

        if staged_only {
            Self::StagedOnly
        } else if unstaged_only {
            Self::UnstagedOnly
        } else {
            Self::StagedAndUnstaged
        }
    }
}

impl<MainIndexer> Workspace<MainIndexer>
where
    MainIndexer: BasicIndexer + Sync,
{
    /// Create a new workspace pointing at the given directory and using the
    /// given configuration.
    ///
    /// The workspace must not already exist.
    pub async fn init(
        repository_index: impl RepositoryIndex,
        repository_name: &RepositoryName,
        provider: Arc<Provider>,
        main_indexer: MainIndexer,
    ) -> Result<Self> {
        let workspace = Self::new(
            repository_index,
            repository_name,
            provider,
            "main",
            main_indexer,
        )
        .await?;

        workspace.initial_checkout().await?;

        Ok(workspace)
    }

    /// Load an existing workspace at the specified location.
    ///
    /// This method expect the target folder to be the root of an existing workspace.
    ///
    /// To load a workspace from a possible subfolder, use `Workspace::find`.
    pub async fn load(
        repository_index: impl RepositoryIndex,
        repository_name: &RepositoryName,
        branch_name: &str,
        provider: Arc<Provider>,
        main_indexer: MainIndexer,
    ) -> Result<Self> {
        Self::new(
            repository_index,
            repository_name,
            provider,
            branch_name,
            main_indexer,
        )
        .await
    }

    /// Return the repository name of the workspace.
    pub fn repository_name(&self) -> &RepositoryName {
        self.index.repository_name()
    }

    /// Returns the name of the source control branch that is active in the workspace.
    pub fn branch_name(&self) -> &str {
        self.branch_name.as_str()
    }

    pub fn id(&self) -> &ContentId {
        &self.content_id
    }

    async fn new(
        repository_index: impl RepositoryIndex,
        repository_name: &RepositoryName,
        provider: Arc<Provider>,
        branch_name: &str,
        main_indexer: MainIndexer,
    ) -> Result<Self> {
        let index = repository_index.load_repository(repository_name).await?;
        let branch = index.get_branch(branch_name).await?;
        let commit = index.get_commit(branch.head).await?;

        Ok(Self {
            index,
            transaction: provider.begin_transaction_in_memory(),
            branch_name: branch_name.to_owned(),
            main_indexer,
            path_indexer: StringPathIndexer::default(),
            content_id: ContentId {
                main_index_tree_id: commit.main_index_tree_id,
                path_index_tree_id: commit.path_index_tree_id,
            },
        })
    }

    /// Get the commits chain, starting from the specified commit.
    pub async fn list_commits<'q>(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        self.index.list_commits(query).await
    }

    /// Get the current commit.
    pub async fn get_current_commit(&self) -> Result<Commit> {
        let current_branch = self.get_current_branch().await?;

        self.index.get_commit(current_branch.head).await
    }

    /// Get the current transaction/provider
    pub fn get_provider(&self) -> &Provider {
        &self.transaction
    }

    async fn get_resource_identifier(&self, id: &IndexKey) -> Result<Option<ResourceIdentifier>> {
        self.get_resource_identifier_from_index(&self.main_indexer, self.get_main_index_id(), id)
            .await
    }

    pub async fn get_resource_identifier_by_path(
        &self,
        path: &str,
    ) -> Result<Option<ResourceIdentifier>> {
        self.get_resource_identifier_from_index(
            &self.path_indexer,
            &self.content_id.path_index_tree_id,
            &path.into(),
        )
        .await
    }

    async fn get_resource_identifier_from_index(
        &self,
        indexer: &impl BasicIndexer,
        tree_id: &TreeIdentifier,
        id: &IndexKey,
    ) -> Result<Option<ResourceIdentifier>> {
        let leaf_node = indexer.get_leaf(&self.transaction, tree_id, id).await;

        match leaf_node {
            Ok(leaf_node) => match leaf_node {
                Some(leaf_node) => match leaf_node {
                    TreeLeafNode::Resource(resource_id) => Ok(Some(resource_id)),
                    TreeLeafNode::TreeRoot(tree_id) => Err(Error::CorruptedIndex { tree_id }),
                },
                None => Ok(None),
            },
            Err(e) => {
                #[cfg(feature = "verbose")]
                {
                    error!(
                        "failed to find resource '{}' in index '{}': {}",
                        id, tree_id, e,
                    );
                    self.dump_all_indices(None).await;
                }

                Err(Error::ContentStoreIndexing(e))
            }
        }
    }

    pub async fn resource_exists(&self, id: &IndexKey) -> Result<bool> {
        let resource_id = self.get_resource_identifier(id).await?;
        Ok(resource_id.is_some())
    }

    pub async fn resource_exists_by_path(&self, path: &str) -> Result<bool> {
        let resource_id = self.get_resource_identifier_by_path(path).await?;
        Ok(resource_id.is_some())
    }

    pub async fn load_resource(&self, id: &IndexKey) -> Result<(Vec<u8>, ResourceIdentifier)> {
        if let Some(resource_id) = self
            .get_resource_identifier_from_index(&self.main_indexer, self.get_main_index_id(), id)
            .await?
        {
            match self
                .transaction
                .read_resource::<ResourceByteReader>(&resource_id)
                .await
            {
                Ok(resource_bytes) => {
                    #[cfg(feature = "verbose")]
                    info!("reading resource '{}' -> {}", id.to_hex(), resource_id);
                    Ok((resource_bytes.into_vec(), resource_id))
                }
                Err(e) => {
                    #[cfg(feature = "verbose")]
                    {
                        error!("failed to read resource '{}': {}", resource_id, e,);
                        self.dump_all_indices(Some(&resource_id)).await;
                    }

                    Err(Error::ContentStoreIndexing(e))
                }
            }
        } else {
            Err(Error::ResourceNotFound { id: id.clone() })
        }
    }

    pub async fn get_resources_for_commit(
        &self,
        commit: &Commit,
    ) -> Result<Vec<(IndexKey, ResourceIdentifier)>> {
        self.get_resources_from_main_by_id(&commit.main_index_tree_id)
            .await
    }

    pub async fn get_resources(&self) -> Result<Vec<(IndexKey, ResourceIdentifier)>> {
        self.get_resources_from_main_by_id(self.get_main_index_id())
            .await
    }

    pub fn get_main_index_id(&self) -> &TreeIdentifier {
        &self.content_id.main_index_tree_id
    }

    pub async fn get_resources_from_main_by_id(
        &self,
        tree_id: &TreeIdentifier,
    ) -> Result<Vec<(IndexKey, ResourceIdentifier)>> {
        self.get_resources_by_index_and_id(&self.main_indexer, tree_id)
            .await
    }

    async fn get_resources_by_index_and_id(
        &self,
        indexer: &(impl BasicIndexer + Sync),
        tree_id: &TreeIdentifier,
    ) -> Result<Vec<(IndexKey, ResourceIdentifier)>> {
        indexing::enumerate_resources(&self.transaction, indexer, tree_id)
            .await
            .map_err(Error::ContentStoreIndexing)
    }

    #[cfg(feature = "verbose")]
    async fn dump_index<F>(
        &self,
        indexer: &(impl BasicIndexer + Sync),
        tree_id: &TreeIdentifier,
        resource_id: Option<&ResourceIdentifier>,
        f: F,
    ) where
        F: Fn(&IndexKey) -> String,
    {
        if let Ok(contents) = self.get_resources_by_index_and_id(indexer, tree_id).await {
            match resource_id {
                Some(resource_id) => {
                    if let Some((index_key, resource_id)) = contents
                        .iter()
                        .find(|(_index_key, match_resource_id)| resource_id == match_resource_id)
                    {
                        info!("index: {}, [{}] -> {}", tree_id, f(index_key), resource_id);
                    }
                }
                None => {
                    info!("contents of index '{}'", tree_id);
                    for (index_key, resource_id) in contents {
                        info!("[{}] -> {}", f(&index_key), resource_id);
                    }
                }
            }
        }
    }

    #[cfg(feature = "verbose")]
    async fn dump_all_indices(&self, resource_id: Option<&ResourceIdentifier>) {
        self.dump_index(
            &self.main_indexer,
            self.get_main_index_id(),
            resource_id,
            IndexKey::to_hex,
        )
        .await;
        self.dump_index(
            &self.path_indexer,
            &self.content_id.path_index_tree_id,
            resource_id,
            |index_key| std::str::from_utf8(index_key.as_ref()).unwrap().to_owned(),
        )
        .await;
    }

    pub async fn reverse_lookup(&self, resource_id: &ResourceIdentifier) -> Result<IndexKey> {
        self.reverse_lookup_in_tree_id(&self.main_indexer, self.get_main_index_id(), resource_id)
            .await
    }

    async fn reverse_lookup_in_tree_id(
        &self,
        indexer: &(impl BasicIndexer + Sync),
        tree_id: &TreeIdentifier,
        resource_id: &ResourceIdentifier,
    ) -> Result<IndexKey> {
        if let Some((index_key, _matched_id)) = self
            .get_resources_by_index_and_id(indexer, tree_id)
            .await?
            .iter()
            .find(|(_index_key, matched_id)| resource_id == matched_id)
        {
            Ok(index_key.clone())
        } else {
            Err(Error::ReverseLookup {
                id: resource_id.clone(),
            })
        }
    }

    /// Add a resource to the local changes.
    ///
    /// The list of new resources added is returned. If all the resources were already
    /// added, an empty list is returned and call still succeeds.
    pub async fn add_resource(
        &mut self,
        id: &IndexKey,
        path: &str,
        contents: &[u8],
    ) -> Result<ResourceIdentifier> {
        let resource_identifier = self
            .transaction
            .write_resource(&ResourceByteWriter::new(contents))
            .await
            .map_err(Error::ContentStoreIndexing)?;

        self.content_id.main_index_tree_id = self
            .main_indexer
            .add_leaf(
                &self.transaction,
                self.get_main_index_id(),
                id,
                TreeLeafNode::Resource(resource_identifier.clone()),
            )
            .await
            .map_err(Error::ContentStoreIndexing)?;

        self.content_id.path_index_tree_id = self
            .path_indexer
            .add_leaf(
                &self.transaction,
                &self.content_id.path_index_tree_id,
                &path.into(),
                TreeLeafNode::Resource(resource_identifier.clone()),
            )
            .await
            .map_err(Error::ContentStoreIndexing)?;

        #[cfg(feature = "verbose")]
        {
            info!(
                "adding resource '{}', path: '{}' -> {}",
                id.to_hex(),
                path,
                resource_identifier,
            );
            self.dump_all_indices(Some(&resource_identifier)).await;
        }

        Ok(resource_identifier)
    }

    pub async fn update_resource(
        &mut self,
        id: &IndexKey,
        path: &str,
        contents: &[u8],
        old_identifier: &ResourceIdentifier,
    ) -> Result<ResourceIdentifier> {
        let resource_identifier = self
            .transaction
            .write_resource(&ResourceByteWriter::new(contents))
            .await
            .map_err(Error::ContentStoreIndexing)?;

        if &resource_identifier != old_identifier {
            // content has changed
            #[cfg(feature = "verbose")]
            {
                info!(
                    "updating resource '{}', path: '{}' -> {}...",
                    id.to_hex(),
                    path,
                    old_identifier,
                );
                self.dump_all_indices(Some(old_identifier)).await;
            }

            // update indices
            let (main_index_tree_id, _leaf_node) = self
                .main_indexer
                .replace_leaf(
                    &self.transaction,
                    self.get_main_index_id(),
                    id,
                    TreeLeafNode::Resource(resource_identifier.clone()),
                )
                .await
                .map_err(Error::ContentStoreIndexing)?;
            self.content_id.main_index_tree_id = main_index_tree_id;

            let (path_index_tree_id, _leaf_node) = self
                .path_indexer
                .replace_leaf(
                    &self.transaction,
                    &self.content_id.path_index_tree_id,
                    &path.into(),
                    TreeLeafNode::Resource(resource_identifier.clone()),
                )
                .await
                .map_err(Error::ContentStoreIndexing)?;
            self.content_id.path_index_tree_id = path_index_tree_id;

            // unwrite previous resource content from content-store
            self.transaction.unwrite_resource(old_identifier).await?;

            #[cfg(feature = "verbose")]
            {
                info!(
                    "... to resource '{}', path: '{}' -> {}",
                    id.to_hex(),
                    path,
                    resource_identifier,
                );
                self.dump_all_indices(Some(&resource_identifier)).await;
            }
        }

        Ok(resource_identifier)
    }

    pub async fn update_path(
        &mut self,
        old_path: &str,
        new_path: &str,
        resource_identifier: &ResourceIdentifier,
    ) -> Result<()> {
        assert!(new_path != old_path);

        assert!(self
            .transaction
            .resource_exists(resource_identifier)
            .await
            .unwrap());

        let (path_index_tree_id, _old_node) = self
            .path_indexer
            .remove_leaf(
                &self.transaction,
                &self.content_id.path_index_tree_id,
                &old_path.into(),
            )
            .await
            .map_err(Error::ContentStoreIndexing)?;
        self.content_id.path_index_tree_id = path_index_tree_id;

        self.content_id.path_index_tree_id = self
            .path_indexer
            .add_leaf(
                &self.transaction,
                &self.content_id.path_index_tree_id,
                &new_path.into(),
                TreeLeafNode::Resource(resource_identifier.clone()),
            )
            .await
            .map_err(Error::ContentStoreIndexing)?;

        #[cfg(feature = "verbose")]
        {
            info!(
                "updating path '{}' (was '{}') -> {}",
                new_path, old_path, resource_identifier,
            );
            self.dump_all_indices(Some(resource_identifier)).await;
        }

        assert!(self
            .transaction
            .resource_exists(resource_identifier)
            .await
            .unwrap());

        Ok(())
    }

    /// Mark some local files for deletion.
    ///
    /// The list of new files edited is returned. If all the files were already
    /// edited, an empty list is returned and call still succeeds.
    pub async fn delete_resource(&mut self, id: &IndexKey) -> Result<()> {
        // remove from main index
        let (main_index_tree_id, leaf_node) = self
            .main_indexer
            .remove_leaf(&self.transaction, self.get_main_index_id(), id)
            .await
            .map_err(Error::ContentStoreIndexing)?;
        self.content_id.main_index_tree_id = main_index_tree_id;

        if let TreeLeafNode::Resource(resource_id) = leaf_node {
            // reverse lookup in path index
            let path = self
                .reverse_lookup_in_tree_id(
                    &self.path_indexer,
                    &self.content_id.path_index_tree_id,
                    &resource_id,
                )
                .await?;

            let (path_index_tree_id, _leaf_node) = self
                .path_indexer
                .remove_leaf(
                    &self.transaction,
                    &self.content_id.path_index_tree_id,
                    &path,
                )
                .await
                .map_err(Error::ContentStoreIndexing)?;
            self.content_id.path_index_tree_id = path_index_tree_id;

            // unwrite resource from content-store
            self.transaction.unwrite_resource(&resource_id).await?;

            #[cfg(feature = "verbose")]
            {
                info!(
                    "deleting resource '{}', path: '{}' -> {}",
                    id.to_hex(),
                    std::str::from_utf8(path.as_ref()).unwrap(),
                    resource_id,
                );
                self.dump_all_indices(Some(&resource_id)).await;
            }
        } else {
            return Err(Error::CorruptedIndex {
                tree_id: self.get_main_index_id().clone(),
            });
        }

        Ok(())
    }

    /*
    /// Returns the status of the workspace, according to the staging
    /// preference.
    pub async fn status(
        &self,
        staging: Staging,
    ) -> Result<(
        BTreeMap<CanonicalPath, Change>,
        BTreeMap<CanonicalPath, Change>,
    )> {
        Ok(match staging {
            Staging::StagedAndUnstaged => (
                self.get_staged_changes().await?,
                self.get_unstaged_changes().await?,
            ),
            Staging::StagedOnly => (self.get_staged_changes().await?, BTreeMap::new()),
            Staging::UnstagedOnly => (BTreeMap::new(), self.get_unstaged_changes().await?),
        })
    }
    */

    /*
    /// Revert local changes to files and unstage them.
    ///
    /// The list of reverted files is returned. If none of the files had changes
    /// - staged or not - an empty list is returned and call still succeeds.
    pub async fn revert_files(
        &self,
        paths: impl IntoIterator<Item = &Path> + Clone,
        staging: Staging,
    ) -> Result<BTreeSet<CanonicalPath>> {
        debug!(
            "revert_files: {}",
            paths
                .clone()
                .into_iter()
                .map(std::path::Path::display)
                .join(", ")
        );

        let canonical_paths = self.to_canonical_paths(paths).await?;

        let (staged_changes, mut unstaged_changes) = self.status(staging).await?;

        let mut changes_to_clear = vec![];
        let mut changes_to_ignore = vec![];
        let mut changes_to_save = vec![];

        let is_selected = |path| -> bool {
            for p in &canonical_paths {
                if p.contains(path) {
                    return true;
                }
            }

            false
        };

        for (canonical_path, staged_change) in
            staged_changes.iter().filter(|(path, _)| is_selected(path))
        {
            match staged_change.change_type() {
                ChangeType::Add { .. } => {
                    changes_to_clear.push(staged_change.clone());
                }
                ChangeType::Edit { old_id, new_id } => {
                    // Only remove local changes if we are not reverting staged changes only.
                    match staging {
                        Staging::UnstagedOnly => {
                            // We should never end-up here.
                            unreachable!();
                        }
                        Staging::StagedOnly => {
                            if new_id != old_id {
                                changes_to_save.push(Change::new(
                                    canonical_path.clone(),
                                    ChangeType::Edit {
                                        old_id: old_id.clone(),
                                        new_id: old_id.clone(),
                                    },
                                ));
                                changes_to_clear.push(staged_change.clone());
                            }
                        }
                        Staging::StagedAndUnstaged => {
                            self.download_file(old_id, canonical_path, Some(true))
                                .await?;
                            changes_to_clear.push(staged_change.clone());

                            // Let's avoid reverting things twice.
                            unstaged_changes.remove(canonical_path);
                        }
                    }
                }
                ChangeType::Delete { old_id } => {
                    self.download_file(old_id, canonical_path, Some(true))
                        .await?;
                    changes_to_clear.push(staged_change.clone());
                }
            }
        }

        for (canonical_path, unstaged_change) in unstaged_changes
            .iter()
            .filter(|(path, _)| is_selected(path))
        {
            match unstaged_change.change_type() {
                ChangeType::Add { .. } => {}
                ChangeType::Edit { old_id, .. } | ChangeType::Delete { old_id } => {
                    let read_only = match staging {
                        Staging::StagedAndUnstaged => Some(true),
                        Staging::StagedOnly => unreachable!(),
                        Staging::UnstagedOnly => None,
                    };

                    self.download_file(old_id, canonical_path, read_only)
                        .await?;
                }
            }

            changes_to_ignore.push(unstaged_change.clone());

            //assert_not_locked(workspace, &abs_path).await?;
        }

        self.backend.clear_staged_changes(&changes_to_clear).await?;
        self.backend.save_staged_changes(&changes_to_save).await?;

        Ok(changes_to_clear
            .into_iter()
            .chain(changes_to_ignore.into_iter())
            .map(Into::into)
            .collect())
    }
    */

    /// Commit the changes in the workspace.
    ///
    /// # Returns
    ///
    /// The commit id.
    pub async fn commit(&mut self, message: &str, behavior: CommitMode) -> Result<Commit> {
        let transaction = std::mem::replace(&mut self.transaction, Provider::new_in_memory());
        let provider = match transaction.commit_transaction().await {
            Ok(provider) => provider,
            Err((provider, e)) => {
                error!("failed to commit to content-store: {}", e);
                provider
            }
        };

        let current_branch = self.get_current_branch().await?;
        let mut branch = self.index.get_branch(&current_branch.name).await?;
        let commit = self.index.get_commit(current_branch.head).await?;

        // Early check in case we are out-of-date long before making the commit.
        if branch.head != current_branch.head {
            return Err(Error::stale_branch(branch));
        }

        let empty_commit = &commit.main_index_tree_id == self.get_main_index_id()
            && commit.path_index_tree_id == self.content_id.path_index_tree_id;

        if empty_commit && matches!(behavior, CommitMode::Strict) {
            return Err(Error::EmptyCommitNotAllowed);
        }

        let commit = if !empty_commit {
            let mut commit = Commit::new_unique_now(
                whoami::username(),
                message,
                commit.changes.clone(),
                self.get_main_index_id().clone(),
                self.content_id.path_index_tree_id.clone(),
                BTreeSet::from([commit.id]),
            );

            commit.id = self.index.commit_to_branch(&commit, &branch).await?;

            branch.head = commit.id;

            commit
        } else {
            commit
        };

        self.transaction = provider.begin_transaction_in_memory();

        Ok(commit)
    }

    /*
    /// Get a list of the currently unstaged changes.
    pub async fn get_unstaged_changes(&self) -> Result<BTreeMap<CanonicalPath, Change>> {
        let commit = self.get_current_commit().await?;
        let staged_changes = self.backend.get_staged_changes().await?;
        let tree = self
            .get_tree_for_commit(&commit, [].into())
            .await?
            .with_changes(staged_changes.values())?;
        let fs_tree = self.get_filesystem_tree([].into()).await?;

        self.get_unstaged_changes_for_trees(&tree, &fs_tree).await
    }
    */

    /*
    /// Get a list of the currently unstaged changes.
    pub async fn get_unstaged_changes_for_trees(
        &self,
        tree: &Tree,
        fs_tree: &Tree,
    ) -> Result<BTreeMap<CanonicalPath, Change>> {
        let mut result = BTreeMap::new();

        for (path, node) in fs_tree.files() {
            if tree.find(&path)?.is_none() {
                let change = Change::new(
                    path.clone(),
                    ChangeType::Add {
                        new_id: node.cs_id().clone(),
                    },
                );

                result.insert(path, change);
            }
        }

        for (path, node) in tree.files() {
            if let Some(Tree::File { id: info, .. }) = fs_tree.find(&path)? {
                if info != node.cs_id() {
                    let change = Change::new(
                        path.clone(),
                        ChangeType::Edit {
                            old_id: node.cs_id().clone(),
                            new_id: info.clone(),
                        },
                    );

                    result.insert(path, change);
                }
            } else {
                let change = Change::new(
                    path.clone(),
                    ChangeType::Delete {
                        old_id: node.cs_id().clone(),
                    },
                );

                result.insert(path, change);
            }
        }

        Ok(result)
    }
    */

    /// Get the current branch.
    pub async fn get_current_branch(&self) -> Result<Branch> {
        self.index.get_branch(self.branch_name.as_str()).await
    }

    /// Create a branch with the given name and the current commit as its head.
    ///
    /// The newly created branch will be a descendant of the current branch and
    /// share the same lock domain.
    pub async fn create_branch(&mut self, branch_name: &str) -> Result<Branch> {
        let current_branch = self.get_current_branch().await?;

        if branch_name == current_branch.name {
            return Err(Error::already_on_branch(current_branch.name));
        }

        let new_branch = current_branch.branch_out(branch_name.to_owned());

        self.index.insert_branch(&new_branch).await?;
        self.branch_name = new_branch.name.clone();

        Ok(new_branch)
    }

    /// Detach the current branch from its parent.
    ///
    /// If the branch is already detached, an error is returned.
    ///
    /// The resulting branch is detached and now uses its own lock domain.
    pub async fn detach_branch(&self) -> Result<Branch> {
        let mut current_branch = self.get_current_branch().await?;

        current_branch.detach();

        self.index.insert_branch(&current_branch).await?;

        Ok(current_branch)
    }

    /// Attach the current branch to the specified branch.
    ///
    /// If the branch is already attached to the specified branch, this is a no-op.
    ///
    /// The resulting branch is attached and now uses the same lock domain as
    /// its parent.
    pub async fn attach_branch(&self, branch_name: &str) -> Result<Branch> {
        let mut current_branch = self.get_current_branch().await?;
        let parent_branch = self.index.get_branch(branch_name).await?;

        current_branch.attach(&parent_branch);

        self.index.insert_branch(&current_branch).await?;
        // self.backend.set_current_branch(&current_branch).await?;

        Ok(current_branch)
    }

    /// Get the branches in the repository.
    pub async fn get_branches(&self) -> Result<BTreeSet<Branch>> {
        Ok(self
            .index
            .list_branches(&ListBranchesQuery::default())
            .await?
            .into_iter()
            .collect())
    }

    /// Switch to a different branch and updates the current files.
    ///
    /// Returns the commit id of the new branch as well as the changes.
    pub async fn switch_branch(&self, _branch_name: &str) -> Result<(Branch, BTreeSet<Change>)> {
        /*
        let current_branch = self.get_current_branch().await?;

        if branch_name == current_branch.name {
            return Err(Error::already_on_branch(branch_name.to_string()));
        }

        let from_commit = self.index.get_commit(current_branch.head).await?;
        let from = self.get_tree_for_commit(&from_commit, [].into()).await?;
        let branch = self.index.get_branch(branch_name).await?;
        let to_commit = self.index.get_commit(branch.head).await?;
        let to = self.get_tree_for_commit(&to_commit, [].into()).await?;

        let changes = self.sync_tree(&from, &to).await?;

        self.backend.set_current_branch(&branch).await?;

        Ok((branch, changes))
        */
        Err(Error::Unspecified("todo".to_owned()))
    }

    /// Sync the current branch to its latest commit.
    ///
    /// # Returns
    ///
    /// The commit id that the workspace was synced to as well as the changes.
    pub async fn sync(&self) -> Result<(Branch, BTreeSet<Change>)> {
        let current_branch = self.get_current_branch().await?;

        let changes = self.sync_to(current_branch.head).await?;

        Ok((current_branch, changes))
    }

    /// Sync the current branch with the specified commit.
    ///
    /// # Returns
    ///
    /// The changes.
    pub async fn sync_to(&self, _commit_id: CommitId) -> Result<BTreeSet<Change>> {
        /*
        let mut current_branch = self.get_current_branch().await?;

        if current_branch.head == commit_id {
            return Ok([].into());
        }

        let from_commit = self.index.get_commit(current_branch.head).await?;
        let from = self.get_tree_for_commit(&from_commit, [].into()).await?;
        let to_commit = self.index.get_commit(commit_id).await?;
        let to = self.get_tree_for_commit(&to_commit, [].into()).await?;

        let changes = self.sync_tree(&from, &to).await?;

        current_branch.head = commit_id;

        self.backend.set_current_branch(&current_branch).await?;

        Ok(changes)
        */
        Err(Error::Unspecified("todo".to_owned()))
    }

    /// Cache a file to the blob storage cache
    ///
    /// # Returns
    ///
    /// The hash of the file.
    /*
    async fn upload_file(&self, canonical_path: &CanonicalPath) -> Result<Identifier> {
        debug!("caching blob for: {}", canonical_path);

        let contents = tokio::fs::read(canonical_path.to_path_buf(&self.root))
            .await
            .map_other_err(format!("failed to read `{}`", &canonical_path))?;

        self.provider
            .write(&contents)
            .await
            .map_other_err(format!("failed to cache file `{}`", canonical_path))
    }
    */

    /*
    async fn download_file(
        &self,
        id: &Identifier,
        path: &CanonicalPath,
        read_only: Option<bool>,
    ) -> Result<()> {
        let abs_path = path.to_path_buf(&self.root);

        match self.provider.get_reader(id).await {
            Ok(mut reader) => {
                let mut f = tokio::fs::File::create(&abs_path)
                    .await
                    .map_other_err(format!("failed to create `{}`", abs_path.display()))?;

                tokio::io::copy(&mut reader, &mut f)
                    .await
                    .map_other_err(format!("failed to write file `{}`", abs_path.display()))?;

                debug!("downloaded blob `{}` from cache", id);

                if let Some(read_only) = read_only {
                    self.make_file_read_only(abs_path, read_only).await
                } else {
                    Ok(())
                }
            }
            Err(err) => Err(err).map_other_err("failed to download blob"),
        }
    }
    */

    async fn initial_checkout(&self) -> Result<()> {
        // 1. Write initial empty indices to content store
        self.transaction.write_tree(&Tree::default()).await.unwrap();

        // 2. Read the head commit information.
        let _commit = self.get_current_commit().await?;

        Ok(())
    }

    /*
    async fn sync_tree(&self, from: &Tree, to: &Tree) -> Result<BTreeSet<Change>> {
        let changes_to_apply = from.get_changes_to(to);

        // Little optimization: no point in computing all that if we know we are
        // coming from an empty tree.
        if !from.is_empty() {
            let staged_changes = self.get_staged_changes().await?;
            let unstaged_changes = self.get_unstaged_changes().await?;

            let conflicting_changes = changes_to_apply
                .iter()
                .filter_map(|change| {
                    staged_changes
                        .get(change.canonical_path())
                        .or_else(|| unstaged_changes.get(change.canonical_path()))
                })
                .cloned()
                .collect::<BTreeSet<_>>();

            if !conflicting_changes.is_empty() {
                return Err(Error::conflicting_changes(conflicting_changes));
            }

            // Process deletions and edits first.
            for change in &changes_to_apply {
                match change.change_type() {
                    ChangeType::Delete { .. } | ChangeType::Edit { .. } => {
                        self.remove_file(change.canonical_path()).await?;
                    }
                    ChangeType::Add { .. } => {}
                };
            }
        }

        // Process additions and edits.
        for change in &changes_to_apply {
            match change.change_type() {
                ChangeType::Add { new_id } | ChangeType::Edit { new_id, .. } => {
                    let abs_path = change.canonical_path().to_path_buf(&self.root);

                    if let Some(parent_abs_path) = abs_path.parent() {
                        tokio::fs::create_dir_all(&parent_abs_path)
                            .await
                            .map_other_err(format!(
                                "failed to create directory at `{}`",
                                parent_abs_path.display()
                            ))?;
                    }

                    // TODO: If the file is an empty directory, replace it.

                    let mut reader =
                        self.provider
                            .get_reader(&new_id)
                            .await
                            .map_other_err(format!(
                                "failed to download blob `{}` to {}",
                                new_id,
                                abs_path.display()
                            ))?;

                    let mut writer =
                        tokio::fs::File::create(&abs_path)
                            .await
                            .map_other_err(format!(
                                "failed to create file at `{}`",
                                abs_path.display()
                            ))?;

                    tokio::io::copy(&mut reader, &mut writer)
                        .await
                        .map_other_err(format!(
                            "failed to write file at `{}`",
                            abs_path.display()
                        ))?;

                    self.make_file_read_only(&abs_path, true).await?;
                }
                ChangeType::Delete { .. } => {}
            };
        }

        Ok(changes_to_apply)
    }
    */

    /*
    async fn remove_file(&self, path: &CanonicalPath) -> Result<()> {
        let abs_path = path.to_path_buf(&self.root);

        // On Windows, one must make the file read-write to be able to delete it.
        #[cfg(target_os = "windows")]
        self.make_file_read_only(&abs_path, false).await?;

        tokio::fs::remove_file(abs_path)
            .await
            .map_other_err(format!("failed to delete file `{}`", path))
    }
    */

    /*
    /// Download a blob from the index backend and write it to the local
    /// temporary folder.
    pub async fn download_temporary_file(&self, id: &Identifier) -> Result<tempfile::TempPath> {
        let temp_file_path = Self::get_tmp_path(&self.root).join(id.to_string());

        let mut reader = self
            .provider
            .get_reader(id)
            .await
            .map_other_err("failed to download blob")?;
        let mut f = tokio::fs::File::create(&temp_file_path)
            .await
            .map_other_err(format!("failed to create `{}`", temp_file_path.display()))?;

        tokio::io::copy(&mut reader, &mut f)
            .await
            .map_other_err(format!(
                "failed to write file `{}`",
                temp_file_path.display()
            ))?;

        Ok(tempfile::TempPath::from_path(temp_file_path))
    }
    */
}
