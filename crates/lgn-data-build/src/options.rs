use std::{path::Path, sync::Arc};

use lgn_content_store::Provider;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::Project;
use lgn_data_runtime::{manifest::Manifest, AssetRegistry};
use lgn_source_control::RepositoryIndex;

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
/// # use std::sync::Arc;
/// # use lgn_data_build::DataBuildOptions;
/// # use lgn_content_store::{Provider, ProviderConfig};
/// # use lgn_data_offline::resource::Project;
/// # use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
/// # tokio_test::block_on(async {
/// let source_control_content_provider = Arc::new(Provider::new_in_memory());
/// let data_content_provider = Arc::new(Provider::new_in_memory());
/// let project = Project::open("project/", source_control_content_provider).await.unwrap();
/// let build = DataBuildOptions::new("temp/".to_string(), data_content_provider, CompilerRegistryOptions::local_compilers("./"))
///         .create(&project).await.unwrap();
/// # })
/// ```
pub struct DataBuildOptions {
    pub(crate) data_content_provider: Arc<Provider>,
    pub(crate) compiler_options: CompilerRegistryOptions,
    pub(crate) registry: Option<Arc<AssetRegistry>>,
    pub(crate) manifest: Option<Manifest>,
}

impl DataBuildOptions {
    /// Create new instance of `DataBuildOptions` with the mandatory options.
    pub fn new(
        data_content_provider: Arc<Provider>,
        compiler_options: CompilerRegistryOptions,
    ) -> Self {
        Self {
            data_content_provider,
            compiler_options,
            registry: None,
            manifest: None,
        }
    }

    /// Set asset registry used by data compilers. If it is not set `DataBuild` will use
    /// a new instance of asset registry.
    #[must_use]
    pub fn asset_registry(mut self, registry: Arc<AssetRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Set manifest used by the asset registry during data compilation.
    #[must_use]
    pub fn manifest(mut self, manifest: Manifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    /// Create new build index for a specified project.
    ///
    /// `project_dir` must be either an absolute path or path relative to
    /// `buildindex_dir`.
    pub async fn create_with_project(
        self,
        project_dir: impl AsRef<Path>,
        repository_index: impl RepositoryIndex,
        source_control_content_provider: Arc<Provider>,
    ) -> Result<(DataBuild, Project), Error> {
        let project = Project::open(
            project_dir,
            repository_index,
            source_control_content_provider,
        )
        .await
        .map_err(Error::from)?;
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
    /// The content store must exist for this to work.
    pub async fn open(self, project: &Project) -> Result<DataBuild, Error> {
        DataBuild::open(self, project).await
    }

    /// Create new build index for a specified project.
    pub async fn create(self, project: &Project) -> Result<DataBuild, Error> {
        DataBuild::new(self, project).await
    }
}
