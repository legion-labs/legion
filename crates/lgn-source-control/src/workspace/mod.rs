use std::path::{Path, PathBuf};

use lgn_blob_storage::BlobStorage;
use serde::{Deserialize, Serialize};

use crate::{
    make_path_absolute, new_index_backend,
    utils::{make_file_read_only, parse_url_or_path},
    Error, IndexBackend, MapOtherError, Result, TreeNode, WorkspaceRegistration,
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
}

impl Workspace {
    /// Create a new workspace pointing at the given directory and using the
    /// given configuration.
    ///
    /// The workspace must not already exist.
    pub async fn init(root: impl AsRef<Path>, config: WorkspaceConfig) -> Result<Self> {
        let root = make_path_absolute(root).map_other_err("failed to make path absolute")?;
        let lsc_directory = Self::get_lsc_directory(&root);

        tokio::fs::create_dir_all(&lsc_directory)
            .await
            .map_other_err("failed to create `.lsc` directory")?;

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
        workspace.checkout("main").await?;

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
    /// If the path is a file, it's parent folder is considered for the
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

        Ok(Self {
            root,
            index_backend,
            blob_storage,
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

    async fn register(&self) -> Result<()> {
        self.index_backend
            .register_workspace(&self.registration)
            .await
    }

    async fn checkout(&self, branch_name: &str) -> Result<()> {
        // 1. Read the branch information.
        let branch = self.index_backend.read_branch(branch_name).await?;

        // 2. Mark the branch as the current branch in the workspace backend.
        self.backend
            .set_current_branch(&branch.name, &branch.head)
            .await?;

        // 3. Read the head commit information.
        let commit = self.index_backend.read_commit(&branch.head).await?;

        // 4. Write the files on disk.
        self.checkout_tree(&commit.root_hash).await
    }

    async fn checkout_tree(&self, tree_hash: &str) -> Result<()> {
        let mut directories_to_process = Vec::from([TreeNode {
            name: String::from(self.root.to_str().expect("invalid workspace path")),
            hash: String::from(tree_hash),
        }]);

        while let Some(dir_node) = directories_to_process.pop() {
            let tree = self.index_backend.read_tree(&dir_node.hash).await?;

            for relative_subdir_node in tree.directory_nodes {
                let abs_subdir_node = TreeNode {
                    name: format!("{}/{}", &dir_node.name, relative_subdir_node.name),
                    hash: relative_subdir_node.hash,
                };

                tokio::fs::create_dir_all(&abs_subdir_node.name)
                    .await
                    .map_other_err("faled to create directory")?;

                directories_to_process.push(abs_subdir_node);
            }

            for relative_file_node in tree.file_nodes {
                let abs_path = PathBuf::from(&dir_node.name).join(relative_file_node.name);

                self.blob_storage
                    .download_blob(&abs_path, &relative_file_node.hash)
                    .await
                    .map_other_err(format!(
                        "failed to download blob `{}` to {}",
                        &relative_file_node.hash,
                        abs_path.display()
                    ))?;

                make_file_read_only(&abs_path, true)
                    .map_other_err("failed to make file read-only")?;
            }
        }

        Ok(())
    }

    pub async fn download_temp_file(&self, blob_hash: &str) -> Result<tempfile::TempPath> {
        let temp_file_path =
            Self::get_tmp_path(Self::get_lsc_directory(&self.root)).join(blob_hash);

        self.blob_storage
            .download_blob(&temp_file_path, blob_hash)
            .await
            .map_other_err("failed to download blob")?;

        Ok(tempfile::TempPath::from_path(temp_file_path))
    }

    fn get_lsc_directory(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(".lsc")
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
    pub registration: WorkspaceRegistration,
    pub index_url: String,
}
