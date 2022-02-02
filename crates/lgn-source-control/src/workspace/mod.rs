use itertools::Itertools;
use lgn_tracing::{debug, warn};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use lgn_blob_storage::{BlobStorage, LocalBlobStorage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    make_path_absolute, new_index_backend, utils::parse_url_or_path, CanonicalPath, Change,
    ChangeType, Commit, Error, IndexBackend, MapOtherError, Result, Tree, TreeFilter,
    WorkspaceRegistration,
};

mod backend;
mod local_backend;

pub use backend::WorkspaceBackend;
pub use local_backend::LocalWorkspaceBackend;

/// Represents a workspace.
pub struct Workspace {
    pub(crate) root: PathBuf,
    pub index_backend: Box<dyn IndexBackend>,
    pub(crate) blob_storage: Box<dyn BlobStorage>,
    pub(crate) backend: Box<dyn WorkspaceBackend>,
    pub(crate) registration: WorkspaceRegistration,
    cache_blob_storage: LocalBlobStorage,
}

impl Workspace {
    const LSC_DIR_NAME: &'static str = ".lsc";

    /// Create a new workspace pointing at the given directory and using the
    /// given configuration.
    ///
    /// The workspace must not already exist.
    pub async fn init(root: impl AsRef<Path>, config: WorkspaceConfig) -> Result<Self> {
        let root = make_path_absolute(root).map_other_err("failed to make path absolute")?;
        let lsc_directory = Self::get_lsc_directory(&root);

        tokio::fs::create_dir_all(&lsc_directory)
            .await
            .map_other_err(format!(
                "failed to create `{}` directory",
                Self::LSC_DIR_NAME
            ))?;

        let blobs_cache_path = Self::get_blobs_cache_path(&lsc_directory);

        tokio::fs::create_dir_all(&blobs_cache_path)
            .await
            .map_other_err("failed to create blobs cache directory")?;

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
        workspace.initial_checkout("main").await?;

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
        let path: &Path =
            &make_path_absolute(path).map_other_err("failed to make path absolute")?;

        let mut path = match tokio::fs::metadata(path).await {
            Ok(metadata) => {
                if metadata.is_dir() {
                    path
                } else {
                    path.parent().ok_or_else(|| Error::not_a_workspace(path))?
                }
            }
            Err(err) => match err.kind() {
                // If the path doesn't exist, assume we specified a file that
                // may not exist but still continue the search with its parent
                // folder if one exists.
                std::io::ErrorKind::NotFound => {
                    path.parent().ok_or_else(|| Error::not_a_workspace(path))?
                }
                _ => {
                    return Err(Error::Other {
                        source: err.into(),
                        context: format!("failed to read metadata of `{}`", path.display()),
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
                            return Err(err);
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
        let blob_storage = index_backend
            .get_blob_storage_url()
            .await?
            .into_blob_storage()
            .await
            .map_other_err("failed to get blob storage")?;

        let cache_blob_storage =
            LocalBlobStorage::new(Self::get_blobs_cache_path(Self::get_lsc_directory(&root)))
                .await
                .map_other_err("failed to initialize the blob storage cache")?;

        Ok(Self {
            root,
            index_backend,
            blob_storage,
            backend,
            registration: config.registration,
            cache_blob_storage,
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
    pub async fn get_filesystem_tree(
        &self,
        inclusion_rules: BTreeSet<CanonicalPath>,
    ) -> Result<Tree> {
        let tree_filter = TreeFilter {
            inclusion_rules,
            exclusion_rules: [CanonicalPath::new(&format!("/{}", Self::LSC_DIR_NAME))?].into(),
        };

        Tree::from_root(&self.root, &tree_filter).await
    }

    /// Get the tree of files and directories for the current branch and commit.
    pub async fn get_current_tree(&self) -> Result<Tree> {
        let (_, current_commit_id) = self.backend.get_current_branch().await?;
        let commit = self.index_backend.read_commit(&current_commit_id).await?;

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
            .read_tree(&commit.root_tree_id)
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
    pub async fn add_files(
        &self,
        paths: impl IntoIterator<Item = &Path> + Clone,
    ) -> Result<BTreeSet<CanonicalPath>> {
        let canonical_paths = self.to_canonical_paths(paths).await?;
        let fs_tree = self.get_filesystem_tree(canonical_paths.clone()).await?;

        // Also get the current tree to check if the files are already added.
        let (_, current_commit_id) = self.backend.get_current_branch().await?;
        let commit = self.index_backend.read_commit(&current_commit_id).await?;
        let tree = self.get_tree_for_commit(&commit, [].into()).await?;

        let staged_changes = self.get_staged_changes().await?;

        let mut changes_to_save = vec![];

        for (canonical_path, file) in fs_tree.files() {
            let change = if let Some(staged_change) = staged_changes.get(&canonical_path) {
                match staged_change.change_type() {
                    ChangeType::Add { new_hash } => {
                        if new_hash == file.hash() {
                            // The file is already staged as add or edit with the correct hash, nothing to do.
                            continue;
                        }

                        self.cache_blob(&canonical_path).await?;

                        Change::new(
                            canonical_path,
                            ChangeType::Add {
                                new_hash: file.hash().to_string(),
                            },
                        )
                    }
                    ChangeType::Edit { old_hash, new_hash } => {
                        if new_hash == file.hash() {
                            // The file is already staged as add or edit with the correct hash, nothing to do.
                            continue;
                        }

                        self.cache_blob(&canonical_path).await?;

                        Change::new(
                            canonical_path,
                            ChangeType::Edit {
                                old_hash: old_hash.clone(),
                                new_hash: file.hash().to_string(),
                            },
                        )
                    }
                    ChangeType::Delete { old_hash } => {
                        // The file was staged for deletion: replace it with an edit.
                        Change::new(
                            canonical_path,
                            ChangeType::Edit {
                                old_hash: old_hash.clone(),
                                new_hash: file.hash().to_string(),
                            },
                        )
                    }
                }
            } else {
                if let Ok(Some(_)) = tree.find(&canonical_path) {
                    // The file is already in the current tree, nothing to do.
                    continue;
                }

                self.cache_blob(&canonical_path).await?;

                Change::new(
                    canonical_path,
                    ChangeType::Add {
                        new_hash: file.hash().to_string(),
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
    pub async fn edit_files(
        &self,
        paths: impl IntoIterator<Item = &Path> + Clone,
    ) -> Result<BTreeSet<CanonicalPath>> {
        let canonical_paths = self.to_canonical_paths(paths).await?;
        let fs_tree = self.get_filesystem_tree(canonical_paths.clone()).await?;
        let (_, current_commit_id) = self.backend.get_current_branch().await?;
        let commit = self.index_backend.read_commit(&current_commit_id).await?;
        let tree = self.get_tree_for_commit(&commit, canonical_paths).await?;
        let staged_changes = self.get_staged_changes().await?;

        let mut changes_to_save = vec![];

        for (canonical_path, file) in fs_tree.files() {
            let change = if let Some(staged_change) = staged_changes.get(&canonical_path) {
                match staged_change.change_type() {
                    ChangeType::Add { new_hash } => {
                        if new_hash == file.hash() {
                            // The file is already staged as add or edit with the correct hash, nothing to do.
                            continue;
                        }

                        self.cache_blob(&canonical_path).await?;

                        Change::new(
                            canonical_path,
                            ChangeType::Add {
                                new_hash: file.hash().to_string(),
                            },
                        )
                    }
                    ChangeType::Edit { old_hash, new_hash } => {
                        if new_hash == file.hash() {
                            // The file is already staged as add or edit with the correct hash, nothing to do.
                            continue;
                        }

                        self.cache_blob(&canonical_path).await?;

                        Change::new(
                            canonical_path,
                            ChangeType::Edit {
                                old_hash: old_hash.clone(),
                                new_hash: file.hash().to_string(),
                            },
                        )
                    }
                    ChangeType::Delete { old_hash } => {
                        // The file was staged for deletion: replace it with an edit.
                        Change::new(
                            canonical_path,
                            ChangeType::Edit {
                                old_hash: old_hash.clone(),
                                new_hash: file.hash().to_string(),
                            },
                        )
                    }
                }
            } else if let Some(tree_node) = tree.find(&canonical_path)? {
                match tree_node {
                    Tree::Directory { .. } => {
                        // The file is a directory, it cannot be edited.
                        return Err(Error::cannot_edit_directory(canonical_path.clone()));
                    }
                    Tree::File { hash: old_hash, .. } => Change::new(
                        canonical_path,
                        ChangeType::Edit {
                            old_hash: old_hash.clone(),
                            new_hash: file.hash().to_string(),
                        },
                    ),
                }
            } else {
                Change::new(
                    canonical_path,
                    ChangeType::Add {
                        new_hash: file.hash().to_string(),
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
    pub async fn delete_files(
        &self,
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

        let fs_tree = self.get_filesystem_tree(canonical_paths.clone()).await?;

        // Also get the current tree to check if the files actually exist in the tree.
        let (_, current_commit_id) = self.backend.get_current_branch().await?;
        let commit = self.index_backend.read_commit(&current_commit_id).await?;
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
                    ChangeType::Edit { old_hash, .. } => {
                        // The file was staged for edit: staged a deletion instead.

                        changes_to_save.push(Change::new(
                            canonical_path,
                            ChangeType::Delete {
                                old_hash: old_hash.clone(),
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
                            old_hash: file.hash().to_string(),
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

    /// Revert local changes to files and unstage them.
    ///
    /// The list of reverted files is returned. If none of the files had changes
    /// - staged or not - an empty list is returned and call still succeeds.
    pub async fn revert_files(
        &self,
        paths: impl IntoIterator<Item = &Path> + Clone,
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

        let staged_changes = self.get_staged_changes().await?;
        let unstaged_changes = self.get_unstaged_changes().await?;

        let mut changes_to_clear = vec![];
        let mut changes_to_ignore = vec![];

        let is_selected = |path| -> bool {
            for p in &canonical_paths {
                if p.matches(path) {
                    return true;
                }
            }

            false
        };

        for (canonical_path, staged_change) in
            staged_changes.iter().filter(|(path, _)| is_selected(path))
        {
            match staged_change.change_type() {
                ChangeType::Add { .. } => {}
                ChangeType::Edit { old_hash, .. } | ChangeType::Delete { old_hash } => {
                    self.download_blob(old_hash, canonical_path).await?;
                }
            }

            changes_to_clear.push(staged_change.clone());
        }

        for (canonical_path, unstaged_change) in unstaged_changes
            .iter()
            .filter(|(path, _)| is_selected(path))
        {
            match unstaged_change.change_type() {
                ChangeType::Add { .. } => {}
                ChangeType::Edit { old_hash, .. } | ChangeType::Delete { old_hash } => {
                    self.download_blob(old_hash, canonical_path).await?;
                }
            }

            changes_to_ignore.push(unstaged_change.clone());

            //assert_not_locked(workspace, &abs_path).await?;
        }

        self.backend.clear_staged_changes(&changes_to_clear).await?;

        Ok(changes_to_clear
            .into_iter()
            .chain(changes_to_ignore.into_iter())
            .map(Into::into)
            .collect())
    }

    pub async fn commit(&self, message: &str) -> Result<()> {
        let fs_tree = self.get_filesystem_tree([].into()).await?;

        let (current_branch_name, current_commit_id) = self.backend.get_current_branch().await?;
        let mut branch = self.index_backend.read_branch(&current_branch_name).await?;
        let commit = self.index_backend.read_commit(&current_commit_id).await?;

        // Early check in case we are out-of-date long before making the commit.
        if branch.head != current_commit_id {
            return Err(Error::stale_branch(branch));
        }

        let staged_changes = self.backend.get_staged_changes().await?;

        //for change in &staged_changes {
        //    assert_not_locked(workspace, &abs_path).await?;
        //}

        // Upload all the data straight away.
        //
        // If this fails, no need to go further and the worst that happens is
        // that we "waste" some storage space.
        let blob_hashes = Self::get_blob_hashes_from_changes(staged_changes.values());
        self.upload_blobs(blob_hashes).await?;

        let tree = self
            .get_tree_for_commit(&commit, [].into())
            .await?
            .with_changes(staged_changes.values())?;
        let tree_id = tree.id();

        if commit.root_tree_id == tree_id {
            return Err(Error::EmptyCommitNotAllowed);
        }

        // Let's update the new tree already.
        self.index_backend
            .save_tree(&tree)
            .await
            .map_other_err("failed to save tree")?;

        let mut parent_commits = BTreeSet::from([current_commit_id]);

        for pending_branch_merge in self.backend.read_pending_branch_merges().await? {
            parent_commits.insert(pending_branch_merge.head.clone());
        }

        let staged_changes = staged_changes.into_values().collect::<BTreeSet<_>>();
        let commit = Commit::new_unique_now(
            whoami::username(),
            message,
            staged_changes.clone(),
            tree_id,
            parent_commits,
        );

        self.index_backend
            .commit_to_branch(&commit, &branch)
            .await?;

        branch.head = commit.id.clone();

        self.backend
            .set_current_branch(&current_branch_name, &commit.id)
            .await?;

        // For all the changes that we commited, we make the files read-only
        // again and release locks, unless said files have unstaged changes on
        // disk.
        for change in &staged_changes {
            match change.change_type() {
                ChangeType::Add { new_hash } | ChangeType::Edit { new_hash, .. } => {
                    if let Some(node) = fs_tree.find(change.canonical_path())? {
                        if node.hash() == new_hash {
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
                        }
                    }
                }
                ChangeType::Delete { .. } => {}
            }
        }

        // Clear the blob cache of all related blobs.
        //
        // This is very likely wrong as it doesn't perfectly clear the cache:
        // - Some blobs could be referenced multiple times and be deleted incorrectly.
        // - Some blobs could be not referenced but will stay in the cache forever.
        //
        // Good enough for now.
        for change in &staged_changes {
            if let Some(new_hash) = change.change_type().new_hash() {
                if let Err(err) = self.uncache_blob(new_hash).await {
                    warn!("failed to uncache blob `{}`: {}", new_hash, err);
                }
            }
        }

        self.backend
            .clear_staged_changes(&staged_changes.into_iter().collect::<Vec<_>>())
            .await?;

        self.backend.clear_pending_branch_merges().await
    }

    /// Get a list of the currently unstaged changes.
    pub async fn get_unstaged_changes(&self) -> Result<BTreeMap<CanonicalPath, Change>> {
        let (_, current_commit_id) = self.backend.get_current_branch().await?;
        let commit = self.index_backend.read_commit(&current_commit_id).await?;
        let staged_changes = self.backend.get_staged_changes().await?;
        let tree = self
            .get_tree_for_commit(&commit, [].into())
            .await?
            .with_changes(staged_changes.values())?;
        let fs_tree = self.get_filesystem_tree([].into()).await?;

        let mut result = BTreeMap::new();

        for (path, node) in fs_tree.files() {
            if tree.find(&path)?.is_none() {
                let change = Change::new(
                    path.clone(),
                    ChangeType::Add {
                        new_hash: node.hash().to_string(),
                    },
                );

                result.insert(path, change);
            }
        }

        for (path, node) in tree.files() {
            if let Some(Tree::File { hash, .. }) = fs_tree.find(&path)? {
                if hash != node.hash() {
                    let change = Change::new(
                        path.clone(),
                        ChangeType::Edit {
                            old_hash: node.hash().to_string(),
                            new_hash: hash.to_string(),
                        },
                    );

                    result.insert(path, change);
                }
            } else {
                let change = Change::new(
                    path.clone(),
                    ChangeType::Delete {
                        old_hash: node.hash().to_string(),
                    },
                );

                result.insert(path, change);
            }
        }

        Ok(result)
    }

    /// Checkout a different branch and updates the current files.
    pub async fn checkout(&self, branch_name: &str) -> Result<()> {
        let (current_branch_name, _current_commit_id) = self.backend.get_current_branch().await?;

        if branch_name == current_branch_name {
            return Err(Error::already_on_branch(branch_name.to_string()));
        }

        let staged_changes = self.backend.get_staged_changes().await?;

        if !staged_changes.is_empty() {
            return Err(Error::WorkspaceDirty);
        }

        let branch = self.index_backend.read_branch(branch_name).await?;
        let commit = self.index_backend.read_commit(&branch.head).await?;

        let tree = self.get_tree_for_commit(&commit, [].into()).await?;
        let fs_tree = self.get_filesystem_tree([].into()).await?;

        println!("{:?}", tree);
        println!("{:?}", fs_tree);

        Ok(())
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

    fn get_blob_hashes_from_changes<'c>(
        staged_changes: impl IntoIterator<Item = &'c Change>,
    ) -> BTreeSet<&'c str> {
        staged_changes
            .into_iter()
            .filter_map(|change| match &change.change_type() {
                ChangeType::Add { new_hash } => Some(new_hash.as_str()),
                ChangeType::Edit { old_hash, new_hash } => {
                    if old_hash != new_hash {
                        Some(new_hash.as_str())
                    } else {
                        None
                    }
                }
                ChangeType::Delete { .. } => None,
            })
            .collect()
    }

    async fn upload_blobs<'c>(&self, blob_hashes: impl IntoIterator<Item = &'c str>) -> Result<()> {
        let futures = blob_hashes.into_iter().map(|hash| self.upload_blob(hash));

        futures::future::join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|_| ())
    }

    async fn get_file_hash(&self, canonical_path: &CanonicalPath) -> Result<(String, Vec<u8>)> {
        let local_path = canonical_path.to_path_buf(&self.root);

        let contents = tokio::fs::read(&local_path)
            .await
            .map_other_err(format!("failed to read `{}`", local_path.display()))?;

        let hash = format!("{:x}", Sha256::digest(&contents));

        Ok((hash, contents))
    }

    /// Cache a file to the blob storage cache
    ///
    /// # Returns
    ///
    /// The hash of the file.
    async fn cache_blob(&self, canonical_path: &CanonicalPath) -> Result<String> {
        debug!("caching blob for: {}", canonical_path);

        let (hash, contents) = self.get_file_hash(canonical_path).await?;

        self.cache_blob_storage
            .write_blob(&hash, &contents)
            .await
            .map_other_err(format!(
                "failed to cache file `{}` as blob `{}`",
                canonical_path, hash
            ))
            .map(|_| hash)
    }

    /// Remove a file from the blob storage cache.
    ///
    /// If the file doesn't exist in the cache, this is a no-op.
    async fn uncache_blob(&self, hash: &str) -> Result<()> {
        debug!("removing cached blob: {}", hash);

        self.cache_blob_storage
            .delete_blob(hash)
            .await
            .map_other_err(format!("failed to remove blob `{}` from the cache", hash))
    }

    /// Upload a file to the blob storage from the blob cache.
    async fn upload_blob(&self, hash: &str) -> Result<()> {
        debug!("uploading blob: {}", hash);

        // FIXME: This would be more efficient if the blob storage API was fully
        // supporting AsyncStreams.

        if self
            .blob_storage
            .blob_exists(hash)
            .await
            .map_other_err("failed to check blob existence")?
        {
            return Ok(());
        }

        let contents = self
            .cache_blob_storage
            .read_blob(hash)
            .await
            .map_other_err("failed to read from blob storage cache")?;

        self.blob_storage
            .write_blob(hash, &contents)
            .await
            .map_other_err(format!("failed to write blob `{}`", hash))
    }

    async fn download_blob(&self, hash: &str, path: &CanonicalPath) -> Result<()> {
        let abs_path = path.to_path_buf(&self.root);

        match self.cache_blob_storage.download_blob(&abs_path, hash).await {
            Ok(()) => {
                debug!("downloaded blob `{}` from cache", hash);

                self.make_file_read_only(abs_path, true).await
            }
            Err(lgn_blob_storage::Error::NoSuchBlob(_)) => match self
                .blob_storage
                .download_blob(&abs_path, hash)
                .await
                .map_other_err("failed to download blob")
            {
                Ok(()) => {
                    debug!("downloaded blob `{}` from blob storage", hash);

                    self.make_file_read_only(abs_path, true).await
                }
                Err(e) => Err(e),
            },
            Err(err) => Err(err).map_other_err("failed to download blob from cache"),
        }
    }

    async fn register(&self) -> Result<()> {
        self.index_backend
            .register_workspace(&self.registration)
            .await
    }

    async fn initial_checkout(&self, branch_name: &str) -> Result<()> {
        // 1. Read the branch information.
        let branch = self.index_backend.read_branch(branch_name).await?;

        // 2. Mark the branch as the current branch in the workspace backend.
        self.backend
            .set_current_branch(&branch.name, &branch.head)
            .await?;

        // 3. Read the head commit information.
        let commit = self.index_backend.read_commit(&branch.head).await?;

        // 4. Read the tree.
        let tree = self.index_backend.read_tree(&commit.root_tree_id).await?;

        // 5. Write the files on disk.
        self.checkout_tree(None, &tree).await
    }

    async fn checkout_tree(&self, from: Option<&Tree>, to: &Tree) -> Result<()> {
        for (path, node) in to {
            let abs_path = path.to_path_buf(&self.root);

            match node {
                Tree::Directory { .. } => {
                    tokio::fs::create_dir_all(&abs_path)
                        .await
                        .map_other_err(format!(
                            "failed to create directory at `{}`",
                            abs_path.display()
                        ))?;
                }
                Tree::File { hash, .. } => {
                    self.blob_storage
                        .download_blob(&abs_path, hash)
                        .await
                        .map_other_err(format!(
                            "failed to download blob `{}` to {}",
                            &hash,
                            abs_path.display()
                        ))?;

                    self.make_file_read_only(&abs_path, true).await?;
                }
            }
        }

        // If a source is provided, remove all the files that were in the source
        // but are not in the destination anymore.
        if let Some(from) = from {
            for (path, _) in from {
                if let Ok(None) = to.find(&path) {
                    self.remove_file(&path).await?;
                }
            }
        }

        Ok(())
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
    pub async fn download_temporary_file(&self, blob_hash: &str) -> Result<tempfile::TempPath> {
        let temp_file_path =
            Self::get_tmp_path(Self::get_lsc_directory(&self.root)).join(blob_hash);

        self.blob_storage
            .download_blob(&temp_file_path, blob_hash)
            .await
            .map_other_err("failed to download blob")?;

        Ok(tempfile::TempPath::from_path(temp_file_path))
    }

    fn get_lsc_directory(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(Self::LSC_DIR_NAME)
    }

    fn get_workspace_config_path(lsc_root: impl AsRef<Path>) -> PathBuf {
        lsc_root.as_ref().join("workspace.json")
    }

    fn get_blobs_cache_path(lsc_root: impl AsRef<Path>) -> PathBuf {
        lsc_root.as_ref().join("blob_cache")
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
