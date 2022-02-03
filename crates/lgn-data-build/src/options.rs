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
/// ```no_run
/// # use lgn_data_build::DataBuildOptions;
/// # use lgn_content_store::ContentStoreAddr;
/// # use lgn_data_offline::resource::Project;
/// # use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
/// # tokio_test::block_on(async {
/// let project = Project::open("project/").await.unwrap();
/// let build = DataBuildOptions::new(".", CompilerRegistryOptions::from_dir("./"))
///         .content_store(&ContentStoreAddr::from("./content_store/"))
///         .create(&project).await.unwrap();
/// # })
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

    /// `projectindex_path` is either absolute or relative to `buildindex_dir`.
    fn construct_project_path(
        buildindex_dir: &Path,
        projectindex_path: &Path,
    ) -> Result<PathBuf, Error> {
        let project_path = if projectindex_path.is_absolute() {
            projectindex_path.to_owned()
        } else {
            buildindex_dir.join(projectindex_path)
        };

        if !project_path.exists() {
            Err(Error::InvalidProject(project_path))
        } else {
            Ok(project_path)
        }
    }

    /// Create new build index for a specified project.
    ///
    /// `project_dir` must be either an absolute path or path relative to
    /// `buildindex_dir`.
    pub async fn create_with_project(
        self,
        project_dir: impl AsRef<Path>,
    ) -> Result<(DataBuild, Project), Error> {
        let projectindex_path = Project::root_to_index_path(project_dir);
        let corrected_path =
            Self::construct_project_path(&self.buildindex_dir, &projectindex_path)?;

        let project = Project::open(corrected_path).await.map_err(Error::from)?;
        let build = DataBuild::new(self, &project).await?;
        Ok((build, project))
    }

    /// Opens the existing build index.
    ///
    /// If the build index does not exist it creates one.
    pub async fn open_or_create(self, project: &Project) -> Result<DataBuild, Error> {
        DataBuild::open_or_create(self, project).await
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
        DataBuild::open(self, project).await
    }

    /// Create new build index for a specified project.
    pub async fn create(self, project: &Project) -> Result<DataBuild, Error> {
        DataBuild::new(self, project).await
    }
}
