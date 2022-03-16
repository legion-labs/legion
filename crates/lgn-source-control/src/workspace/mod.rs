use itertools::Itertools;
use lgn_content_store2::{ChunkIdentifier, Chunker, ContentProvider};
use lgn_tracing::{debug, warn};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    make_path_absolute, new_index_backend, utils::parse_url_or_path, Branch, CanonicalPath, Change,
    ChangeType, Commit, CommitId, Error, IndexBackend, ListBranchesQuery, ListCommitsQuery,
    MapOtherError, Result, Tree, TreeFilter, WorkspaceRegistration,
};

mod backend;
mod local_backend;

pub use backend::WorkspaceBackend;
pub use local_backend::LocalWorkspaceBackend;

/// Represents a workspace.
pub struct Workspace {
    pub(crate) root: PathBuf,
    pub index_backend: Box<dyn IndexBackend>,
    pub(crate) backend: Box<dyn WorkspaceBackend>,
    pub(crate) registration: WorkspaceRegistration,
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

impl Workspace {
    const LSC_DIR_NAME: &'static str = ".lsc";

    /// Create a new workspace pointing at the given directory and using the
    /// given configuration.
    ///
    /// The workspace must not already exist.
    pub async fn init<CP: ContentProvider + Send + Sync>(
        root: impl AsRef<Path>,
        config: WorkspaceConfig,
        content_store_provider: &Chunker<CP>,
    ) -> Result<Self> {
        let root = make_path_absolute(root).map_other_err("failed to make path absolute")?;
        let lsc_directory = Self::get_lsc_directory(&root);

        tokio::fs::create_dir_all(&lsc_directory)
            .await
            .map_other_err(format!(
                "failed to create `{}` directory",
                Self::LSC_DIR_NAME
            ))?;

        let tmp_path = Self::get_tmp_path(&lsc_directory);

        tokio::fs::create_dir_all(&tmp_path)
            .await
            .map_other_err("failed to create tmp directory")?;

        let workspace_config_path = Self::get_workspace_config_path(&lsc_directory);

        let config_data = serde_json::to_string(&config)
            .map_other_err("failed to serialize workspace configuration")?;

        tokio::fs::write(workspace_config_path, config_data)
            .await
            .map_other_err("failed to write workspace configuration")?;

        let backend = Box::new(LocalWorkspaceBackend::create(lsc_directory).await?);

        let workspace = Self::new(root, config, backend).await?;

        workspace.register().await?;
        workspace
            .initial_checkout(content_store_provider, "main")
            .await?;

        Ok(workspace)
    }

    /// Load an existing workspace at the specified location.
    ///
    /// This method expect the target folder to be the root of an existing workspace.
    ///
    /// To load a workspace from a possible subfolder, use `Workspace::find`.
    pub async fn load(root: impl AsRef<Path>) -> Result<Self> {
        let root = make_path_absolute(root).map_other_err("failed to make path absolute")?;
        let lsc_directory = Self::get_lsc_directory(&root);
        let workspace_config_path = Self::get_workspace_config_path(lsc_directory);

        let config_data = match tokio::fs::read_to_string(workspace_config_path).await {
            Ok(data) => data,
            Err(err) => {
                return match err.kind() {
                    std::io::ErrorKind::NotFound => Err(Error::not_a_workspace(root)),
                    _ => Err(Error::Other {
                        source: err.into(),
                        context: format!(
                            "failed to read workspace configuration in `{}`",
                            root.display()
                        ),
                    }),
                };
            }
        };

        let config: WorkspaceConfig = serde_json::from_str(&config_data)
            .map_other_err("failed to parse workspace configuration")?;

        let lsc_directory = Self::get_lsc_directory(&root);
        let backend = Box::new(LocalWorkspaceBackend::connect(lsc_directory).await?);

        Self::new(root, config, backend).await
    }

    /// Find an existing workspace in the specified folder or one of its
    /// parents, recursively.
    ///
    /// If the path is a file, its parent folder is considered for the
    /// first lookup.
    ///
    /// The method stops whenever a workspace is found or when it reaches the
    /// root folder, whichever comes first.
    pub async fn find(path: impl AsRef<Path>) -> Result<Self> {
        let initial_path: &Path =
            &make_path_absolute(path).map_other_err("failed to make path absolute")?;

        let mut path = match tokio::fs::metadata(initial_path).await {
            Ok(metadata) => {
                if metadata.is_dir() {
                    initial_path
                } else {
                    initial_path
                        .parent()
                        .ok_or_else(|| Error::not_a_workspace(initial_path))?
                }
            }
            Err(err) => match err.kind() {
                // If the path doesn't exist, assume we specified a file that
                // may not exist but still continue the search with its parent
                // folder if one exists.
                std::io::ErrorKind::NotFound => initial_path
                    .parent()
                    .ok_or_else(|| Error::not_a_workspace(initial_path))?,
                _ => {
                    return Err(Error::Other {
                        source: err.into(),
                        context: format!("failed to read metadata of `{}`", initial_path.display()),
                    })
                }
            },
        };

        loop {
            match Self::load(path).await {
                Ok(workspace) => return Ok(workspace),
                Err(err) => match err {
                    Error::NotAWorkspace { path: _ } => {
                        if let Some(parent_path) = path.parent() {
                            path = parent_path;
                        } else {
                            return Err(Error::not_a_workspace(initial_path));
                        }
                    }
                    _ => return Err(err),
                },
            }
        }
    }

    fn try_make_filepath_absolute(url: &str, root: &Path) -> Result<String> {
        match parse_url_or_path(url)
            .map_other_err(format!("failed to parse index url `{}`", &url))?
        {
            crate::utils::UrlOrPath::Url(_) => Ok(url.to_owned()),
            crate::utils::UrlOrPath::Path(path) => {
                if path.is_absolute() {
                    Ok(url.to_owned())
                } else {
                    Ok(root.join(path).into_os_string().into_string().unwrap())
                }
            }
        }
    }

    async fn new(
        root: PathBuf,
        config: WorkspaceConfig,
        backend: Box<dyn WorkspaceBackend>,
    ) -> Result<Self> {
        let absolute_url = Self::try_make_filepath_absolute(&config.index_url, &root)?;
        let index_backend = new_index_backend(&absolute_url)?;

        Ok(Self {
            root,
            index_backend,
            backend,
            registration: config.registration,
        })
    }

    /// Find an existing workspace in the current directory.
    ///
    /// This is a convenience method that calls `Workspace::find` with the
    /// current working directory.
    pub async fn find_in_current_directory() -> Result<Self> {
        Self::find(std::env::current_dir().map_other_err("failed to determine current directory")?)
            .await
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the canonical paths designated by the specified paths.
    ///
    /// If the path is not absolute, it is assumed to be relative to the workspace root.
    ///
    /// If the path points to a file outside the workspace, an error is returned.
    ///
    /// If a path to a directory is specified, all the files and subdirectories
    /// under it are considered.
    ///
    /// If the path points to a symbolic link, an error is returned.
    pub async fn to_canonical_paths(
        &self,
        paths: impl IntoIterator<Item = &Path> + Clone,
    ) -> Result<BTreeSet<CanonicalPath>> {
        let root = tokio::fs::canonicalize(&self.root)
            .await
            .map_other_err(format!(
                "failed to canonicalize root `{}`",
                self.root.display()
            ))?;

        futures::future::join_all(
            paths
                .into_iter()
                .map(|path| CanonicalPath::new_from_canonical_root(&root, path)),
        )
        .await
        .into_iter()
        .collect::<Result<BTreeSet<_>>>()
    }

    /// Get the tree of files and directories for the disk.
    pub async fn get_filesystem_tree<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        inclusion_rules: BTreeSet<CanonicalPath>,
    ) -> Result<Tree> {
        let tree_filter = TreeFilter {
            inclusion_rules,
            exclusion_rules: [CanonicalPath::new(&format!("/{}", Self::LSC_DIR_NAME))?].into(),
        };

        Tree::from_root(content_store_provider, &self.root, &tree_filter).await
    }

    /// Give the current relative path for a given canonical path.
    pub fn make_relative_path(&self, current_dir: &Path, path: &CanonicalPath) -> String {
        let abs_path = path.to_path_buf(&self.root);

        match pathdiff::diff_paths(&abs_path, current_dir) {
            Some(path) => path,
            None => abs_path,
        }
        .display()
        .to_string()
    }

    /// Get the commits chain, starting from the specified commit.
    pub async fn list_commits<'q>(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        self.index_backend.list_commits(query).await
    }

    /// Get the current commit.
    pub async fn get_current_commit(&self) -> Result<Commit> {
        let current_branch = self.backend.get_current_branch().await?;

        self.index_backend.get_commit(current_branch.head).await
    }

    /// Get the tree of files and directories for the current branch and commit.
    pub async fn get_current_tree(&self) -> Result<Tree> {
        let commit = self.get_current_commit().await?;

        self.get_tree_for_commit(&commit, [].into()).await
    }

    /// Get the tree of files and directories for the specified commit.
    async fn get_tree_for_commit(
        &self,
        commit: &Commit,
        inclusion_rules: BTreeSet<CanonicalPath>,
    ) -> Result<Tree> {
        let tree_filter = TreeFilter {
            inclusion_rules,
            exclusion_rules: BTreeSet::new(),
        };

        self.index_backend
            .get_tree(&commit.root_tree_id)
            .await
            .map_other_err(format!(
                "failed to get the current tree at commit {}",
                commit.id
            ))?
            .filter(&tree_filter)
    }

    /// Get the list of staged changes, regardless of the actual content of the
    /// files or their existence on disk or in the current tree.
    pub async fn get_staged_changes(&self) -> Result<BTreeMap<CanonicalPath, Change>> {
        self.backend.get_staged_changes().await
    }

    /// Add files to the local changes.
    ///
    /// The list of new files added is returned. If all the files were already
    /// added, an empty list is returned and call still succeeds.
    pub async fn add_files<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        paths: impl IntoIterator<Item = &Path> + Clone,
    ) -> Result<BTreeSet<CanonicalPath>> {
        let canonical_paths = self.to_canonical_paths(paths).await?;
        let fs_tree = self
            .get_filesystem_tree(content_store_provider, canonical_paths.clone())
            .await?;

        // Also get the current tree to check if the files are already added.
        let tree = self.get_current_tree().await?;

        let staged_changes = self.get_staged_changes().await?;

        let mut changes_to_save = vec![];

        for (canonical_path, file) in fs_tree.files() {
            let change = if let Some(staged_change) = staged_changes.get(&canonical_path) {
                match staged_change.change_type() {
                    ChangeType::Add {
                        new_chunk_id: new_info,
                    } => {
                        let info = file.chunk_id();

                        if new_info == info {
                            // The file is already staged as add or edit with the correct hash, nothing to do.
                            continue;
                        }

                        self.upload_file(content_store_provider, &canonical_path)
                            .await?;

                        Change::new(
                            canonical_path,
                            ChangeType::Add {
                                new_chunk_id: info.clone(),
                            },
                        )
                    }
                    ChangeType::Edit {
                        old_chunk_id: old_info,
                        new_chunk_id: new_info,
                    } => {
                        let info = file.chunk_id();

                        if new_info == info {
                            // The file is already staged as add or edit with the correct hash, nothing to do.
                            continue;
                        }

                        self.upload_file(content_store_provider, &canonical_path)
                            .await?;

                        Change::new(
                            canonical_path,
                            ChangeType::Edit {
                                old_chunk_id: old_info.clone(),
                                new_chunk_id: info.clone(),
                            },
                        )
                    }
                    ChangeType::Delete {
                        old_chunk_id: old_info,
                    } => {
                        // The file was staged for deletion: replace it with an edit.
                        Change::new(
                            canonical_path,
                            ChangeType::Edit {
                                old_chunk_id: old_info.clone(),
                                new_chunk_id: file.chunk_id().clone(),
                            },
                        )
                    }
                }
            } else {
                if let Ok(Some(_)) = tree.find(&canonical_path) {
                    // The file is already in the current tree, nothing to do.
                    continue;
                }

                self.upload_file(content_store_provider, &canonical_path)
                    .await?;

                Change::new(
                    canonical_path,
                    ChangeType::Add {
                        new_chunk_id: file.chunk_id().clone(),
                    },
                )
            };

            //assert_not_locked(workspace, &abs_path).await?;

            changes_to_save.push(change);
        }

        self.backend.save_staged_changes(&changes_to_save).await?;

        Ok(changes_to_save.into_iter().map(Into::into).collect())
    }

    /// Mark some local files for edition.
    ///
    /// The list of new files edited is returned. If all the files were already
    /// edited, an empty list is returned and call still succeeds.
    ///
    /// Calling this method on newly added files is not an error but does
    /// nothing.
    pub async fn checkout_files<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        paths: impl IntoIterator<Item = &Path> + Clone,
    ) -> Result<BTreeSet<CanonicalPath>> {
        let canonical_paths = self.to_canonical_paths(paths).await?;
        let fs_tree = self
            .get_filesystem_tree(content_store_provider, canonical_paths.clone())
            .await?;
        let commit = self.get_current_commit().await?;
        let tree = self.get_tree_for_commit(&commit, canonical_paths).await?;
        let staged_changes = self.get_staged_changes().await?;

        let mut changes_to_save = vec![];

        for (canonical_path, file) in fs_tree.files() {
            if staged_changes.contains_key(&canonical_path) {
                // The file is already staged, nothing to do.
                continue;
            }

            let change = if let Some(tree_node) = tree.find(&canonical_path)? {
                match tree_node {
                    Tree::Directory { .. } => {
                        // The file is a directory, it cannot be edited.
                        return Err(Error::cannot_edit_directory(canonical_path.clone()));
                    }
                    Tree::File { chunk_id: info, .. } => Change::new(
                        canonical_path,
                        ChangeType::Edit {
                            old_chunk_id: info.clone(),
                            new_chunk_id: file.chunk_id().clone(),
                        },
                    ),
                }
            } else {
                // The file is not known to the source-control: assume we mean to add it.
                Change::new(
                    canonical_path,
                    ChangeType::Add {
                        new_chunk_id: file.chunk_id().clone(),
                    },
                )
            };

            //assert_not_locked(workspace, &abs_path).await?;

            changes_to_save.push(change);
        }

        self.backend.save_staged_changes(&changes_to_save).await?;

        for change in &changes_to_save {
            self.make_file_read_only(change.canonical_path().to_path_buf(&self.root), false)
                .await?;
        }

        Ok(changes_to_save.into_iter().map(Into::into).collect())
    }

    /// Mark some local files for deletion.
    ///
    /// The list of new files edited is returned. If all the files were already
    /// edited, an empty list is returned and call still succeeds.
    pub async fn delete_files<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        paths: impl IntoIterator<Item = &Path> + Clone,
    ) -> Result<BTreeSet<CanonicalPath>> {
        debug!(
            "delete_files: {}",
            paths
                .clone()
                .into_iter()
                .map(std::path::Path::display)
                .join(", ")
        );

        let canonical_paths = self.to_canonical_paths(paths).await?;

        let fs_tree = self
            .get_filesystem_tree(content_store_provider, canonical_paths.clone())
            .await?;

        // Also get the current tree to check if the files actually exist in the tree.
        let commit = self.get_current_commit().await?;
        let tree = self.get_tree_for_commit(&commit, canonical_paths).await?;

        let staged_changes = self.get_staged_changes().await?;

        let mut changes_to_save = vec![];
        let mut changes_to_clear = vec![];

        for (canonical_path, _) in fs_tree.files() {
            self.remove_file(&canonical_path).await?;

            if let Some(staged_change) = staged_changes.get(&canonical_path) {
                match staged_change.change_type() {
                    ChangeType::Add { .. } => {
                        // The file was staged for addition: remove the staged change instead.
                        changes_to_clear.push(staged_change.clone());
                    }
                    ChangeType::Edit {
                        old_chunk_id: old_hash,
                        ..
                    } => {
                        // The file was staged for edit: staged a deletion instead.

                        changes_to_save.push(Change::new(
                            canonical_path,
                            ChangeType::Delete {
                                old_chunk_id: old_hash.clone(),
                            },
                        ));
                    }
                    ChangeType::Delete { .. } => {
                        // The file was staged for deletion already: nothing to do.
                        continue;
                    }
                }
            } else {
                // Only stage the deletion if the file is already in the current tree.
                if let Ok(Some(file)) = tree.find(&canonical_path) {
                    changes_to_save.push(Change::new(
                        canonical_path,
                        ChangeType::Delete {
                            old_chunk_id: file.chunk_id().clone(),
                        },
                    ));
                }
            };

            //assert_not_locked(workspace, &abs_path).await?;
        }

        self.backend.clear_staged_changes(&changes_to_clear).await?;
        self.backend.save_staged_changes(&changes_to_save).await?;

        Ok(changes_to_save
            .into_iter()
            .chain(changes_to_clear.into_iter())
            .map(Into::into)
            .collect())
    }

    /// Returns the status of the workspace, according to the staging
    /// preference.
    pub async fn status<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        staging: Staging,
    ) -> Result<(
        BTreeMap<CanonicalPath, Change>,
        BTreeMap<CanonicalPath, Change>,
    )> {
        Ok(match staging {
            Staging::StagedAndUnstaged => (
                self.get_staged_changes().await?,
                self.get_unstaged_changes(content_store_provider).await?,
            ),
            Staging::StagedOnly => (self.get_staged_changes().await?, BTreeMap::new()),
            Staging::UnstagedOnly => (
                BTreeMap::new(),
                self.get_unstaged_changes(content_store_provider).await?,
            ),
        })
    }

    /// Revert local changes to files and unstage them.
    ///
    /// The list of reverted files is returned. If none of the files had changes
    /// - staged or not - an empty list is returned and call still succeeds.
    pub async fn revert_files<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
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

        let (staged_changes, mut unstaged_changes) =
            self.status(content_store_provider, staging).await?;

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
                ChangeType::Edit {
                    old_chunk_id,
                    new_chunk_id,
                } => {
                    // Only remove local changes if we are not reverting staged changes only.
                    match staging {
                        Staging::UnstagedOnly => {
                            // We should never end-up here.
                            unreachable!();
                        }
                        Staging::StagedOnly => {
                            if new_chunk_id != old_chunk_id {
                                changes_to_save.push(Change::new(
                                    canonical_path.clone(),
                                    ChangeType::Edit {
                                        old_chunk_id: old_chunk_id.clone(),
                                        new_chunk_id: old_chunk_id.clone(),
                                    },
                                ));
                                changes_to_clear.push(staged_change.clone());
                            }
                        }
                        Staging::StagedAndUnstaged => {
                            self.download_file(
                                content_store_provider,
                                old_chunk_id,
                                canonical_path,
                                Some(true),
                            )
                            .await?;
                            changes_to_clear.push(staged_change.clone());

                            // Let's avoid reverting things twice.
                            unstaged_changes.remove(canonical_path);
                        }
                    }
                }
                ChangeType::Delete { old_chunk_id } => {
                    self.download_file(
                        content_store_provider,
                        old_chunk_id,
                        canonical_path,
                        Some(true),
                    )
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
                ChangeType::Edit { old_chunk_id, .. } | ChangeType::Delete { old_chunk_id } => {
                    let read_only = match staging {
                        Staging::StagedAndUnstaged => Some(true),
                        Staging::StagedOnly => unreachable!(),
                        Staging::UnstagedOnly => None,
                    };

                    self.download_file(
                        content_store_provider,
                        old_chunk_id,
                        canonical_path,
                        read_only,
                    )
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

    /// Commit the changes in the workspace.
    ///
    /// # Returns
    ///
    /// The commit id.
    pub async fn commit<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        message: &str,
        behavior: CommitMode,
    ) -> Result<Commit> {
        let fs_tree = self
            .get_filesystem_tree(content_store_provider, [].into())
            .await?;

        let current_branch = self.backend.get_current_branch().await?;
        let mut branch = self.index_backend.get_branch(&current_branch.name).await?;
        let commit = self.index_backend.get_commit(current_branch.head).await?;

        // Early check in case we are out-of-date long before making the commit.
        if branch.head != current_branch.head {
            return Err(Error::stale_branch(branch));
        }

        let staged_changes = self.get_staged_changes().await?;

        //for change in &staged_changes {
        //    assert_not_locked(workspace, &abs_path).await?;
        //}

        if matches!(behavior, CommitMode::Strict) {
            let unchanged_files_marked_for_edition: BTreeSet<_> = staged_changes
                .iter()
                .filter_map(|(path, change)| {
                    if change.change_type().has_modifications() {
                        None
                    } else {
                        Some(path.clone())
                    }
                })
                .collect();

            if !unchanged_files_marked_for_edition.is_empty() {
                return Err(Error::unchanged_files_marked_for_edition(
                    unchanged_files_marked_for_edition,
                ));
            }
        }

        let tree = self
            .get_tree_for_commit(&commit, [].into())
            .await?
            .with_changes(staged_changes.values())?;
        let tree_id = tree.id();

        let empty_commit = commit.root_tree_id == tree_id;

        if empty_commit && matches!(behavior, CommitMode::Strict) {
            return Err(Error::EmptyCommitNotAllowed);
        }

        // Let's update the new tree already.
        self.index_backend
            .save_tree(&tree)
            .await
            .map_other_err("failed to save tree")?;

        let mut parent_commits = BTreeSet::from([current_branch.head]);

        for pending_branch_merge in self.backend.read_pending_branch_merges().await? {
            parent_commits.insert(pending_branch_merge.head);
        }

        let staged_changes = staged_changes.into_values().collect::<BTreeSet<_>>();

        let commit = if !empty_commit {
            let mut commit = Commit::new_unique_now(
                whoami::username(),
                message,
                staged_changes.clone(),
                tree_id,
                parent_commits,
            );

            commit.id = self
                .index_backend
                .commit_to_branch(&commit, &branch)
                .await?;

            branch.head = commit.id;

            self.backend.set_current_branch(&branch).await?;
            commit
        } else {
            commit
        };

        let mut changes_to_save = Vec::new();

        // For all the changes that we commited, we make the files read-only
        // again and release locks, unless said files have unstaged changes on
        // disk.
        for change in &staged_changes {
            match change.change_type() {
                ChangeType::Add {
                    new_chunk_id: new_info,
                }
                | ChangeType::Edit {
                    new_chunk_id: new_info,
                    ..
                } => {
                    if let Some(node) = fs_tree.find(change.canonical_path())? {
                        if node.chunk_id() == new_info {
                            if let Err(err) = self
                                .make_file_read_only(
                                    change.canonical_path().to_path_buf(&self.root),
                                    true,
                                )
                                .await
                            {
                                warn!(
                                    "failed to make file `{}` read only: {}",
                                    change.canonical_path(),
                                    err
                                );
                            }
                        } else {
                            // The file has some unstaged changes: we will need
                            // to mark it for edition at the end of the commit.
                            changes_to_save.push(Change::new(
                                change.canonical_path().clone(),
                                ChangeType::Edit {
                                    old_chunk_id: new_info.clone(),
                                    new_chunk_id: new_info.clone(),
                                },
                            ));
                        }
                    }
                }
                ChangeType::Delete { .. } => {}
            }
        }

        self.backend
            .clear_staged_changes(&staged_changes.into_iter().collect::<Vec<_>>())
            .await?;

        self.backend
            .save_staged_changes(changes_to_save.as_slice())
            .await?;

        self.backend.clear_pending_branch_merges().await?;

        Ok(commit)
    }

    /// Get a list of the currently unstaged changes.
    pub async fn get_unstaged_changes<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
    ) -> Result<BTreeMap<CanonicalPath, Change>> {
        let commit = self.get_current_commit().await?;
        let staged_changes = self.backend.get_staged_changes().await?;
        let tree = self
            .get_tree_for_commit(&commit, [].into())
            .await?
            .with_changes(staged_changes.values())?;
        let fs_tree = self
            .get_filesystem_tree(content_store_provider, [].into())
            .await?;

        self.get_unstaged_changes_for_trees(&tree, &fs_tree).await
    }

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
                        new_chunk_id: node.chunk_id().clone(),
                    },
                );

                result.insert(path, change);
            }
        }

        for (path, node) in tree.files() {
            if let Some(Tree::File { chunk_id: info, .. }) = fs_tree.find(&path)? {
                if info != node.chunk_id() {
                    let change = Change::new(
                        path.clone(),
                        ChangeType::Edit {
                            old_chunk_id: node.chunk_id().clone(),
                            new_chunk_id: info.clone(),
                        },
                    );

                    result.insert(path, change);
                }
            } else {
                let change = Change::new(
                    path.clone(),
                    ChangeType::Delete {
                        old_chunk_id: node.chunk_id().clone(),
                    },
                );

                result.insert(path, change);
            }
        }

        Ok(result)
    }

    /// Get the current branch.
    pub async fn get_current_branch(&self) -> Result<Branch> {
        self.backend.get_current_branch().await
    }

    /// Create a branch with the given name and the current commit as its head.
    ///
    /// The newly created branch will be a descendant of the current branch and
    /// share the same lock domain.
    pub async fn create_branch(&self, branch_name: &str) -> Result<Branch> {
        let current_branch = self.backend.get_current_branch().await?;

        if branch_name == current_branch.name {
            return Err(Error::already_on_branch(current_branch.name));
        }

        let old_branch = self.index_backend.get_branch(&current_branch.name).await?;
        let new_branch = old_branch.branch_out(branch_name.to_string());

        self.index_backend.insert_branch(&new_branch).await?;
        self.backend.set_current_branch(&new_branch).await?;

        Ok(new_branch)
    }

    /// Detach the current branch from its parent.
    ///
    /// If the branch is already detached, an error is returned.
    ///
    /// The resulting branch is detached and now uses its own lock domain.
    pub async fn detach_branch(&self) -> Result<Branch> {
        let mut current_branch = self.backend.get_current_branch().await?;

        current_branch.detach();

        self.index_backend.insert_branch(&current_branch).await?;
        self.backend.set_current_branch(&current_branch).await?;

        Ok(current_branch)
    }

    /// Attach the current branch to the specified branch.
    ///
    /// If the branch is already attached to the specified branch, this is a no-op.
    ///
    /// The resulting branch is attached and now uses the same lock domain as
    /// its parent.
    pub async fn attach_branch(&self, branch_name: &str) -> Result<Branch> {
        let mut current_branch = self.backend.get_current_branch().await?;
        let parent_branch = self.index_backend.get_branch(branch_name).await?;

        current_branch.attach(&parent_branch);

        self.index_backend.insert_branch(&current_branch).await?;
        self.backend.set_current_branch(&current_branch).await?;

        Ok(current_branch)
    }

    /// Get the branches in the repository.
    pub async fn get_branches(&self) -> Result<BTreeSet<Branch>> {
        Ok(self
            .index_backend
            .list_branches(&ListBranchesQuery::default())
            .await?
            .into_iter()
            .collect())
    }

    /// Switch to a different branch and updates the current files.
    ///
    /// Returns the commit id of the new branch as well as the changes.
    pub async fn switch_branch<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        branch_name: &str,
    ) -> Result<(Branch, BTreeSet<Change>)> {
        let current_branch = self.backend.get_current_branch().await?;

        if branch_name == current_branch.name {
            return Err(Error::already_on_branch(branch_name.to_string()));
        }

        let from_commit = self.index_backend.get_commit(current_branch.head).await?;
        let from = self.get_tree_for_commit(&from_commit, [].into()).await?;
        let branch = self.index_backend.get_branch(branch_name).await?;
        let to_commit = self.index_backend.get_commit(branch.head).await?;
        let to = self.get_tree_for_commit(&to_commit, [].into()).await?;

        let changes = self.sync_tree(content_store_provider, &from, &to).await?;

        self.backend.set_current_branch(&branch).await?;

        Ok((branch, changes))
    }

    /// Sync the current branch to its latest commit.
    ///
    /// # Returns
    ///
    /// The commit id that the workspace was synced to as well as the changes.
    pub async fn sync<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
    ) -> Result<(Branch, BTreeSet<Change>)> {
        let current_branch = self.backend.get_current_branch().await?;

        let branch = self.index_backend.get_branch(&current_branch.name).await?;

        let changes = self.sync_to(content_store_provider, branch.head).await?;

        Ok((branch, changes))
    }

    /// Sync the current branch with the specified commit.
    ///
    /// # Returns
    ///
    /// The changes.
    pub async fn sync_to<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        commit_id: CommitId,
    ) -> Result<BTreeSet<Change>> {
        let mut current_branch = self.backend.get_current_branch().await?;

        if current_branch.head == commit_id {
            return Ok([].into());
        }

        let from_commit = self.index_backend.get_commit(current_branch.head).await?;
        let from = self.get_tree_for_commit(&from_commit, [].into()).await?;
        let to_commit = self.index_backend.get_commit(commit_id).await?;
        let to = self.get_tree_for_commit(&to_commit, [].into()).await?;

        let changes = self.sync_tree(content_store_provider, &from, &to).await?;

        current_branch.head = commit_id;

        self.backend.set_current_branch(&current_branch).await?;

        Ok(changes)
    }

    async fn make_file_read_only(&self, path: impl AsRef<Path>, readonly: bool) -> Result<()> {
        let path = path.as_ref();

        let metadata = tokio::fs::metadata(&path)
            .await
            .map_other_err(format!("failed to get metadata for {}", path.display()))?;

        let mut permissions = metadata.permissions();

        if permissions.readonly() == readonly {
            return Ok(());
        }

        permissions.set_readonly(readonly);

        tokio::fs::set_permissions(&path, permissions)
            .await
            .map_other_err(format!("failed to set permissions for {}", path.display()))
    }

    /// Cache a file to the blob storage cache
    ///
    /// # Returns
    ///
    /// The hash of the file.
    async fn upload_file<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        canonical_path: &CanonicalPath,
    ) -> Result<ChunkIdentifier> {
        debug!("caching blob for: {}", canonical_path);

        let contents = tokio::fs::read(canonical_path.to_path_buf(&self.root))
            .await
            .map_other_err(format!("failed to read `{}`", &canonical_path))?;

        content_store_provider
            .write_chunk(&contents)
            .await
            .map_other_err(format!("failed to cache file `{}`", canonical_path))
    }

    async fn download_file<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        id: &ChunkIdentifier,
        path: &CanonicalPath,
        read_only: Option<bool>,
    ) -> Result<()> {
        let abs_path = path.to_path_buf(&self.root);

        match content_store_provider.get_chunk_reader(id).await {
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

    async fn register(&self) -> Result<()> {
        self.index_backend
            .register_workspace(&self.registration)
            .await
    }

    async fn initial_checkout<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        branch_name: &str,
    ) -> Result<BTreeSet<Change>> {
        // 1. Read the branch information.
        let branch = self.index_backend.get_branch(branch_name).await?;

        // 2. Mark the branch as the current branch in the workspace backend.
        self.backend.set_current_branch(&branch).await?;

        // 3. Read the head commit information.
        let commit = self.index_backend.get_commit(branch.head).await?;

        // 4. Read the tree.
        let tree = self.index_backend.get_tree(&commit.root_tree_id).await?;

        // 5. Write the files on disk.
        self.sync_tree(content_store_provider, &Tree::empty(), &tree)
            .await
    }

    async fn sync_tree<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        from: &Tree,
        to: &Tree,
    ) -> Result<BTreeSet<Change>> {
        let changes_to_apply = from.get_changes_to(to);

        // Little optimization: no point in computing all that if we know we are
        // coming from an empty tree.
        if !from.is_empty() {
            let staged_changes = self.get_staged_changes().await?;
            let unstaged_changes = self.get_unstaged_changes(content_store_provider).await?;

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
                ChangeType::Add { new_chunk_id } | ChangeType::Edit { new_chunk_id, .. } => {
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

                    let mut reader = content_store_provider
                        .get_chunk_reader(new_chunk_id)
                        .await
                        .map_other_err(format!(
                            "failed to download blob `{}` to {}",
                            new_chunk_id,
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

    async fn remove_file(&self, path: &CanonicalPath) -> Result<()> {
        let abs_path = path.to_path_buf(&self.root);

        // On Windows, one must make the file read-write to be able to delete it.
        #[cfg(target_os = "windows")]
        self.make_file_read_only(&abs_path, false).await?;

        tokio::fs::remove_file(abs_path)
            .await
            .map_other_err(format!("failed to delete file `{}`", path))
    }

    /// Download a blob from the index backend and write it to the local
    /// temporary folder.
    pub async fn download_temporary_file<CP: ContentProvider + Send + Sync>(
        &self,
        content_store_provider: &Chunker<CP>,
        chunk_id: &ChunkIdentifier,
    ) -> Result<tempfile::TempPath> {
        let temp_file_path =
            Self::get_tmp_path(Self::get_lsc_directory(&self.root)).join(chunk_id.to_string());

        let mut reader = content_store_provider
            .get_chunk_reader(chunk_id)
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

    fn get_lsc_directory(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(Self::LSC_DIR_NAME)
    }

    fn get_workspace_config_path(lsc_root: impl AsRef<Path>) -> PathBuf {
        lsc_root.as_ref().join("workspace.json")
    }

    fn get_tmp_path(lsc_root: impl AsRef<Path>) -> PathBuf {
        lsc_root.as_ref().join("tmp")
    }
}

/// Contains the configuration for a workspace.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceConfig {
    index_url: String,
    registration: WorkspaceRegistration,
}

impl WorkspaceConfig {
    pub fn new(index_url: String, registration: WorkspaceRegistration) -> Self {
        Self {
            index_url,
            registration,
        }
    }
}
