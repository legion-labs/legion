use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use lgn_content_store::ContentStoreAddr;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::Project;
use lgn_data_runtime::AssetRegistry;

use crate::{DataBuild, Error};

/// Options and flags used by [`DataBuild`].
///
/// To open or create `DataBuild` first call [`DataBuildOptions::new`], then
/// chain calls to methods to set different options, then call
/// [`DataBuildOptions::open_or_create`]. This will give  you a [`Result`] with
/// a [`DataBuild`] that you can further operate on.
///
/// # Example Usage
///
/// ```
/// # use lgn_data_build::DataBuildOptions;
/// # use lgn_content_store::ContentStoreAddr;
/// # use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
/// let mut build = DataBuildOptions::new(".", CompilerRegistryOptions::from_dir("./"))
///         .content_store(&ContentStoreAddr::from("./content_store/"))
///         .create(".");
/// ```
pub struct DataBuildOptions {
    pub(crate) buildindex_dir: PathBuf,
    pub(crate) contentstore_path: ContentStoreAddr,
    pub(crate) compiler_options: CompilerRegistryOptions,
    pub(crate) registry: Option<Arc<AssetRegistry>>,
}

impl DataBuildOptions {
    /// Creates a new data build options.
    pub fn new(
        buildindex_dir: impl AsRef<Path>,
        compiler_options: CompilerRegistryOptions,
    ) -> Self {
        Self {
            buildindex_dir: buildindex_dir.as_ref().to_owned(),
            contentstore_path: ContentStoreAddr::from(buildindex_dir.as_ref()),
            compiler_options,
            registry: None,
        }
    }

    /// Set content store location for derived resources.
    pub fn content_store(mut self, contentstore_path: &ContentStoreAddr) -> Self {
        self.contentstore_path = contentstore_path.clone();
        self
    }

    /// Set asset registry used by data compilers. If it is not set `DataBuild` will use
    /// a new instance of asset registry.
    pub fn asset_registry(mut self, registry: Arc<AssetRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Opens the existing build index.
    ///
    /// If the build index does not exist it creates one if a project is present
    /// in the same directory.
    ///
    /// `project_dir` must be either an absolute path or path relative to
    /// `buildindex_dir`.
    pub async fn open_or_create_with_project(
        self,
        project_dir: impl AsRef<Path>,
    ) -> Result<(DataBuild, Project), Error> {
        DataBuild::open_or_create(self, project_dir.as_ref()).await
    }

    /// Opens existing build index.
    ///
    /// The following conditions need to be met to successfully open a build
    /// index:
    /// * [`ContentStore`](`lgn_content_store::ContentStore`) must exist under
    ///   address set by [`DataBuildOptions::content_store()`].
    /// * Build index must exist and be of a supported version provided by
    ///   [`DataBuildOptions::new()`].
    /// * The build index must point to an existing
    ///   [`lgn_data_offline::resource::Project`].
    pub async fn open_with_project(self) -> Result<(DataBuild, Project), Error> {
        DataBuild::open(self).await
    }

    /// Create new build index for a specified project.
    ///
    /// `project_dir` must be either an absolute path or path relative to
    /// `buildindex_dir`.
    pub async fn create_with_project(
        self,
        project_dir: impl AsRef<Path>,
    ) -> Result<(DataBuild, Project), Error> {
        DataBuild::new(self, project_dir.as_ref()).await
    }

    /// Opens the existing build index.
    ///
    /// If the build index does not exist it creates one.
    pub async fn open_or_create(self, project: &Project) -> Result<DataBuild, Error> {
        DataBuild::open_or_create_with_proj(self, project).await
    }

    /// Opens existing build index.
    ///
    /// The following conditions need to be met to successfully open a build
    /// index:
    /// * [`ContentStore`](`lgn_content_store::ContentStore`) must exist under
    ///   address set by [`DataBuildOptions::content_store()`].
    /// * Build index must exist and be of a supported version provided by
    ///   [`DataBuildOptions::new()`].
    pub async fn open(self, project: &Project) -> Result<DataBuild, Error> {
        DataBuild::open_with_proj(self, project).await
    }

    /// Create new build index for a specified project.
    pub async fn create(self, project: &Project) -> Result<DataBuild, Error> {
        DataBuild::new_with_proj(self, project).await
    }
}
