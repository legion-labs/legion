use std::{collections::BTreeSet, sync::Arc};

use lgn_content_store::{
    indexing::{
        BasicIndexer, IndexKey, IndexKeyDisplayFormat, ResourceIdentifier, ResourceIndex,
        ResourceReader, ResourceWriter, SharedTreeIdentifier, StringPathIndexer, TreeDiffSide,
        TreeIdentifier, TreeLeafNode,
    },
    ContentAsyncReadWithOriginAndSize, Provider,
};
use tokio_stream::StreamExt;

use crate::{
    Branch, BranchName, ChangeType, Commit, Error, Index, ListBranchesQuery, ListCommitsQuery,
    NewCommit, RepositoryIndex, RepositoryName, Result,
};

/// Represents a workspace.
pub struct Workspace<MainIndexer>
where
    MainIndexer: BasicIndexer + Clone + Sync,
{
    index: Box<dyn Index>,
    transaction: Arc<Provider>,
    branch_name: BranchName,
    main_index: ResourceIndex<MainIndexer>,
    path_index: ResourceIndex<StringPathIndexer>,
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
    MainIndexer: BasicIndexer + Clone + Sync,
{
    /// Load an existing workspace at the specified location.
    ///
    /// This method expect the target folder to be the root of an existing workspace.
    ///
    /// To load a workspace from a possible subfolder, use `Workspace::find`.
    pub async fn new(
        repository_index: impl RepositoryIndex,
        repository_name: &RepositoryName,
        branch_name: &BranchName,
        persistent_provider: Arc<Provider>,
        main_indexer: MainIndexer,
    ) -> Result<Self> {
        let index = repository_index.load_repository(repository_name).await?;
        let branch = index.get_branch(branch_name).await?;
        let commit = index.get_commit(branch_name, branch.head).await?;
        let transaction = Arc::new(persistent_provider.begin_transaction_in_memory());
        let main_index = ResourceIndex::new_shared_with_raw_id(
            Arc::clone(&transaction),
            main_indexer,
            commit.main_index_tree_id,
        );
        let path_index = ResourceIndex::new_exclusive_with_id(
            Arc::clone(&transaction),
            StringPathIndexer::default(),
            commit.path_index_tree_id,
        );
        Ok(Self {
            index,
            transaction,
            branch_name: branch_name.clone(),
            main_index,
            path_index,
        })
    }

    async fn commit_and_restart_transaction(&mut self) -> Result<()> {
        self.transaction
            .commit_and_restart_transaction()
            .await
            .map_err(Error::ContentStore)
    }

    /// Return the repository name of the workspace.
    pub fn repository_name(&self) -> &RepositoryName {
        self.index.repository_name()
    }

    /// Returns the name of the source control branch that is active in the workspace.
    pub fn branch_name(&self) -> &str {
        self.branch_name.as_str()
    }

    pub fn indices(&self) -> (TreeIdentifier, TreeIdentifier) {
        (self.main_index.id(), self.path_index.id())
    }

    /// Get the commits chain, starting from the specified commit.
    pub async fn list_commits<'q>(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        self.index.list_commits(query).await
    }

    /// Get the current commit.
    async fn get_current_commit(&self) -> Result<Commit> {
        let current_branch = self.get_current_branch().await?;

        self.index
            .get_commit(&current_branch.name, current_branch.head)
            .await
    }

    pub async fn get_resource_identifier(
        &self,
        id: &IndexKey,
    ) -> Result<Option<ResourceIdentifier>> {
        self.get_resource_identifier_from_index(&self.main_index, id, IndexKeyDisplayFormat::Hex)
            .await
    }

    pub async fn get_resource_identifier_by_path(
        &self,
        path: &str,
    ) -> Result<Option<ResourceIdentifier>> {
        if path.len() > 1 {
            self.get_resource_identifier_from_index(
                &self.path_index,
                &path.into(),
                IndexKeyDisplayFormat::Utf8,
            )
            .await
        } else {
            // path is invalid, too short
            Err(Error::invalid_path(path))
        }
    }

    #[allow(unused_variables)]
    async fn get_resource_identifier_from_index<Indexer>(
        &self,
        index: &ResourceIndex<Indexer>,
        id: &IndexKey,
        format: IndexKeyDisplayFormat,
    ) -> Result<Option<ResourceIdentifier>>
    where
        Indexer: BasicIndexer + Sync,
    {
        let result = index
            .get_identifier(id)
            .await
            .map_err(Error::ContentStoreIndexing);

        #[cfg(feature = "verbose")]
        {
            print!(
                "\nlookup resource '{}' in index {}",
                id.format(format),
                index.id()
            );
            match &result {
                Ok(Some(resource_id)) => {
                    println!(" -> {}", resource_id);
                }
                Ok(None) => {
                    println!(" -> not found",);
                }
                Err(err) => {
                    println!(": FAILED, {}", err);
                }
            }
        }

        result
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
            .get_resource_identifier_from_index(&self.main_index, id, IndexKeyDisplayFormat::Hex)
            .await?
        {
            let resource_bytes = self.load_resource_by_id(&resource_id).await?;
            #[cfg(feature = "verbose")]
            println!(
                "\nreading resource '{}' -> {}",
                id.format(IndexKeyDisplayFormat::Hex),
                resource_id
            );
            Ok((resource_bytes, resource_id))
        } else {
            Err(Error::resource_not_found_by_id(id.clone()))
        }
    }

    pub async fn get_reader(
        &self,
        id: &IndexKey,
    ) -> Result<(ContentAsyncReadWithOriginAndSize, ResourceIdentifier)> {
        if let Some(resource_id) = self
            .get_resource_identifier_from_index(&self.main_index, id, IndexKeyDisplayFormat::Hex)
            .await?
        {
            if let Ok(reader) = self
                .transaction
                .get_reader(resource_id.as_identifier())
                .await
            {
                return Ok((reader, resource_id));
            }
        }
        Err(Error::ResourceNotFoundById { id: id.clone() })
    }

    pub async fn load_resource_by_path(&self, path: &str) -> Result<(Vec<u8>, ResourceIdentifier)> {
        if let Some(resource_id) = self
            .get_resource_identifier_from_index(
                &self.path_index,
                &path.into(),
                IndexKeyDisplayFormat::Utf8,
            )
            .await?
        {
            let resource_bytes = self.load_resource_by_id(&resource_id).await?;
            #[cfg(feature = "verbose")]
            println!("\nreading resource '{}' -> {}", path, resource_id);
            Ok((resource_bytes, resource_id))
        } else {
            Err(Error::resource_not_found_by_path(path))
        }
    }

    pub async fn load_resource_by_id(&self, resource_id: &ResourceIdentifier) -> Result<Vec<u8>> {
        Ok(self.transaction.read_resource_as_bytes(resource_id).await?)
    }

    pub async fn get_committed_resources(&self) -> Result<Vec<(IndexKey, ResourceIdentifier)>> {
        let commit = self.get_current_commit().await?;
        let commit_manifest = ResourceIndex::new_exclusive_with_id(
            Arc::clone(self.main_index.provider()),
            self.main_index.indexer().clone(),
            commit.main_index_tree_id,
        );
        commit_manifest
            .enumerate_resources()
            .await
            .map_err(Error::ContentStoreIndexing)
    }

    pub async fn get_resources(&self) -> Result<Vec<(IndexKey, ResourceIdentifier)>> {
        self.main_index
            .enumerate_resources()
            .await
            .map_err(Error::ContentStoreIndexing)
    }

    pub fn clone_main_index_id(&self) -> SharedTreeIdentifier {
        self.main_index.shared_id()
    }

    #[cfg(feature = "verbose")]
    async fn dump_index<Indexer>(
        &self,
        index: &ResourceIndex<Indexer>,
        resource_id: Option<&ResourceIdentifier>,
        format: IndexKeyDisplayFormat,
    ) where
        Indexer: BasicIndexer + Sync,
    {
        if let Ok(contents) = index.enumerate_resources().await {
            match resource_id {
                Some(resource_id) => {
                    if let Some((index_key, resource_id)) = contents
                        .iter()
                        .find(|(_index_key, match_resource_id)| resource_id == match_resource_id)
                    {
                        println!(
                            "index: {}, [{}] -> {}",
                            index.id(),
                            index_key.format(format),
                            resource_id
                        );
                    }
                }
                None => {
                    println!("contents of index '{}'", index.id());
                    for (index_key, resource_id) in contents {
                        println!("[{}] -> {}", index_key.format(format), resource_id);
                    }
                }
            }
        }
    }

    #[cfg(feature = "verbose")]
    async fn dump_all_indices(&self, resource_id: Option<&ResourceIdentifier>) {
        self.dump_index(&self.main_index, resource_id, IndexKeyDisplayFormat::Hex)
            .await;
        self.dump_index(&self.path_index, resource_id, IndexKeyDisplayFormat::Utf8)
            .await;
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
        let resource_identifier = self.transaction.write_resource_from_bytes(contents).await?;

        self.main_index
            .add_resource(id, resource_identifier.clone())
            .await?;
        self.path_index
            .add_resource(&path.into(), resource_identifier.clone())
            .await?;

        #[cfg(feature = "verbose")]
        {
            println!(
                "\nadding resource '{}', path: '{}' -> {}",
                id.format(IndexKeyDisplayFormat::Hex),
                path,
                resource_identifier,
            );
            self.dump_all_indices(Some(&resource_identifier)).await;
        }

        self.commit_and_restart_transaction().await?;

        Ok(resource_identifier)
    }

    pub async fn update_resource(
        &mut self,
        id: &IndexKey,
        path: &str,
        contents: &[u8],
        old_identifier: &ResourceIdentifier,
    ) -> Result<ResourceIdentifier> {
        let resource_identifier = self.transaction.write_resource_from_bytes(contents).await?;

        if &resource_identifier != old_identifier {
            // content has changed
            #[cfg(feature = "verbose")]
            {
                println!(
                    "\nupdating resource '{}', path: '{}' -> {}...",
                    id.format(IndexKeyDisplayFormat::Hex),
                    path,
                    old_identifier,
                );
                self.dump_all_indices(Some(old_identifier)).await;
            }

            // update indices
            let replaced_id = self
                .main_index
                .replace_resource(id, resource_identifier.clone())
                .await?;
            assert_eq!(&replaced_id, old_identifier);
            let replaced_id = self
                .path_index
                .replace_resource(&path.into(), resource_identifier.clone())
                .await?;
            assert_eq!(&replaced_id, old_identifier);

            // unwrite previous resource content from content-store
            self.transaction.unwrite_resource(old_identifier).await?;

            #[cfg(feature = "verbose")]
            {
                println!(
                    "... to resource '{}', path: '{}' -> {}",
                    id.format(IndexKeyDisplayFormat::Hex),
                    path,
                    resource_identifier,
                );
                self.dump_all_indices(Some(&resource_identifier)).await;
            }
        }

        self.commit_and_restart_transaction().await?;

        Ok(resource_identifier)
    }

    pub async fn update_resource_and_path(
        &mut self,
        id: &IndexKey,
        old_path: &str,
        new_path: &str,
        contents: &[u8],
        old_identifier: &ResourceIdentifier,
    ) -> Result<ResourceIdentifier> {
        let resource_identifier = self.transaction.write_resource_from_bytes(contents).await?;

        if &resource_identifier != old_identifier {
            // content has changed
            #[cfg(feature = "verbose")]
            {
                println!(
                    "\nrenaming resource '{}', path: '{}' -> {}...",
                    id.format(IndexKeyDisplayFormat::Hex),
                    old_path,
                    old_identifier,
                );
                self.dump_all_indices(Some(old_identifier)).await;
            }

            // update indices
            let replaced_id = self
                .main_index
                .replace_resource(id, resource_identifier.clone())
                .await?;
            assert_eq!(&replaced_id, old_identifier);

            let removed_id = self.path_index.remove_resource(&old_path.into()).await?;
            assert_eq!(&removed_id, old_identifier);

            self.path_index
                .add_resource(&new_path.into(), resource_identifier.clone())
                .await?;

            // unwrite previous resource content from content-store
            self.transaction.unwrite_resource(old_identifier).await?;

            #[cfg(feature = "verbose")]
            {
                println!(
                    "... to resource '{}', path: '{}' -> {}",
                    id.format(IndexKeyDisplayFormat::Hex),
                    new_path,
                    resource_identifier,
                );
                self.dump_all_indices(Some(&resource_identifier)).await;
            }
        }

        self.commit_and_restart_transaction().await?;

        Ok(resource_identifier)
    }

    /// Mark some local files for deletion.
    ///
    /// The list of new files edited is returned. If all the files were already
    /// edited, an empty list is returned and call still succeeds.
    pub async fn delete_resource(
        &mut self,
        id: &IndexKey,
        path: &str,
    ) -> Result<ResourceIdentifier> {
        // remove from main index
        let resource_id = self.main_index.remove_resource(id).await?;

        let removed_id = self.path_index.remove_resource(&path.into()).await?;
        assert_eq!(resource_id, removed_id);

        // unwrite resource from content-store
        self.transaction.unwrite_resource(&resource_id).await?;

        #[cfg(feature = "verbose")]
        {
            println!(
                "\ndeleting resource '{}', path: '{}' -> {}",
                id.format(IndexKeyDisplayFormat::Hex),
                std::str::from_utf8(path.as_ref()).unwrap(),
                resource_id,
            );
            self.dump_all_indices(Some(&resource_id)).await;
        }

        self.commit_and_restart_transaction().await?;

        Ok(resource_id)
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

    pub async fn revert_resource(&self, _id: &IndexKey, _path: &str) -> Result<ResourceIdentifier> {
        Err(Error::Unspecified("todo: revert_resource".to_owned()))
    }

    pub async fn get_pending_changes(&self) -> Result<Vec<(IndexKey, ChangeType)>> {
        let commit_index_id = self.get_current_commit().await?.main_index_tree_id;
        let main_index_id = self.main_index.id();

        let mut leaves = self
            .main_index
            .indexer()
            .diff_leaves(self.main_index.provider(), &commit_index_id, &main_index_id)
            .await?
            .map(|(side, index_key, leaf)| match leaf {
                Ok(leaf) => match leaf {
                    TreeLeafNode::Resource(resource_id) => Ok((index_key, side, resource_id)),
                    TreeLeafNode::TreeRoot(_) => {
                        Err(lgn_content_store::indexing::Error::CorruptedTree(
                            "found unexpected tree-root node".to_owned(),
                        ))
                    }
                },
                Err(err) => Err(err),
            })
            .collect::<Result<Vec<_>, lgn_content_store::indexing::Error>>()
            .await?;
        leaves.sort();
        let mut leaves = leaves.into_iter();

        let mut changes = Vec::new();
        if let Some((mut previous_index_key, mut previous_side, mut previous_resource_id)) =
            leaves.next()
        {
            if previous_side == TreeDiffSide::Right {
                changes.push((
                    previous_index_key.clone(),
                    ChangeType::Add {
                        new_id: previous_resource_id.clone(),
                    },
                ));
            }

            for (index_key, side, resource_id) in leaves.by_ref() {
                if side == TreeDiffSide::Right {
                    if previous_side == TreeDiffSide::Left {
                        // pattern: Left, Right -> Edit | (Delete, Add)
                        if index_key == previous_index_key {
                            // same index-key in both trees
                            changes.push((
                                previous_index_key.clone(),
                                ChangeType::Edit {
                                    old_id: previous_resource_id.clone(),
                                    new_id: resource_id.clone(),
                                },
                            ));
                        } else {
                            // keys don't match, so left entry represents something deleted ...
                            changes.push((
                                previous_index_key.clone(),
                                ChangeType::Delete {
                                    old_id: previous_resource_id.clone(),
                                },
                            ));
                            // ... and right entry something added
                            changes.push((
                                index_key.clone(),
                                ChangeType::Add {
                                    new_id: resource_id.clone(),
                                },
                            ));
                        }
                    } else {
                        // pattern: Right, Right -> (Add | Edit), Add
                        changes.push((
                            index_key.clone(),
                            ChangeType::Add {
                                new_id: resource_id.clone(),
                            },
                        ));
                    }
                } else {
                    // side == TreeDiffSide::Left

                    if previous_side == TreeDiffSide::Left {
                        // pattern: Left, Left -> Delete, (Delete | Edit)
                        changes.push((
                            previous_index_key.clone(),
                            ChangeType::Delete {
                                old_id: previous_resource_id.clone(),
                            },
                        ));
                    } else {
                        // pattern: Right, Left -> (Add | Edit), (Delete | Edit)
                        // do nothing, either was already handled, or will be handled next iteration
                    }
                }

                previous_index_key = index_key;
                previous_side = side;
                previous_resource_id = resource_id;
            }

            if previous_side == TreeDiffSide::Left {
                // ended with left-side, so must be a deletion (since it can't be an edit without matching right side)
                changes.push((
                    previous_index_key,
                    ChangeType::Delete {
                        old_id: previous_resource_id,
                    },
                ));
            }
        }

        #[cfg(feature = "verbose")]
        {
            println!(
                "\npending changes, comparing commit {} with {}",
                commit_index_id, main_index_id
            );
            for (index_key, change_type) in &changes {
                println!("{}, {}", index_key, change_type);
            }
        }

        Ok(changes)
    }

    /// Commit the changes in the workspace.
    ///
    /// # Returns
    ///
    /// The commit id.
    pub async fn commit(&mut self, message: &str, behavior: CommitMode) -> Result<Commit> {
        assert!(!self.transaction.has_references().await);

        let current_branch = self.get_current_branch().await?;
        let mut branch = self.index.get_branch(&current_branch.name).await?;
        let commit = self
            .index
            .get_commit(&current_branch.name, current_branch.head)
            .await?;

        // Early check in case we are out-of-date long before making the commit.
        if branch.head != current_branch.head {
            return Err(Error::stale_branch(branch));
        }

        let empty_commit = commit.main_index_tree_id == self.main_index.id()
            && commit.path_index_tree_id == self.path_index.id();

        if empty_commit && matches!(behavior, CommitMode::Strict) {
            return Err(Error::EmptyCommitNotAllowed);
        }

        let commit = if !empty_commit {
            let new_commit = NewCommit::new_unique_now(
                whoami::username(),
                message,
                self.main_index.id(),
                self.path_index.id(),
                BTreeSet::from([commit.id]),
            );

            let commit = self
                .index
                .commit_to_branch(&branch.name, new_commit)
                .await?;

            branch.head = commit.id;

            commit
        } else {
            commit
        };

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
        self.index.get_branch(&self.branch_name).await
    }

    /// Create a branch with the given name and the current commit as its head.
    ///
    /// The newly created branch will be a descendant of the current branch and
    /// share the same lock domain.
    pub async fn create_branch(&mut self, branch_name: &BranchName) -> Result<Branch> {
        let current_branch = self.get_current_branch().await?;

        if branch_name == &current_branch.name {
            return Err(Error::already_on_branch(current_branch.name));
        }

        let new_branch = current_branch.branch_out(branch_name.clone());

        let branch = self.index.insert_branch(new_branch).await?;
        self.branch_name = branch.name.clone();

        Ok(branch)
    }

    /// Detach the current branch from its parent.
    ///
    /// If the branch is already detached, an error is returned.
    ///
    /// The resulting branch is detached and now uses its own lock domain.
    pub async fn detach_branch(&self) -> Result<Branch> {
        let current_branch = self.get_current_branch().await?;

        let new_branch = current_branch.detach();
        let branch = self.index.insert_branch(new_branch).await?;

        Ok(branch)
    }

    /// Attach the current branch to the specified branch.
    ///
    /// If the branch is already attached to the specified branch, this is a no-op.
    ///
    /// The resulting branch is attached and now uses the same lock domain as
    /// its parent.
    pub async fn attach_branch(&self, branch_name: &BranchName) -> Result<Branch> {
        let current_branch = self.get_current_branch().await?;
        let parent_branch = self.index.get_branch(branch_name).await?;

        let new_branch = current_branch.attach(&parent_branch);
        let branch = self.index.insert_branch(new_branch).await?;
        // self.backend.set_current_branch(&current_branch).await?;

        Ok(branch)
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

    /*
    /// Switch to a different branch and updates the current files.
    ///
    /// Returns the commit id of the new branch as well as the changes.
    pub async fn switch_branch(&self, _branch_name: &BranchName) -> Result<(Branch, BTreeSet<Change>)> {
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
        Err(Error::Unspecified("todo".to_owned()))
    }
    */

    /*
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
    */

    /*
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
    */

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
