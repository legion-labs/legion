use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use lgn_content_store::Provider;
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
    pub(crate) source_control_content_provider: Arc<Provider>,
    pub(crate) data_content_provider: Arc<Provider>,
    pub(crate) output_db_addr: String,
    pub(crate) compiler_options: CompilerRegistryOptions,
    pub(crate) registry: Option<Arc<AssetRegistry>>,
}

impl DataBuildOptions {
    /// Creates a new data build options.
    pub fn new_with_sqlite_output(
        output_dir: impl AsRef<Path>,
        compiler_options: CompilerRegistryOptions,
        source_control_content_provider: Arc<Provider>,
        data_content_provider: Arc<Provider>,
    ) -> Self {
        assert!(output_dir.as_ref().is_absolute());
        let output_db_addr = Self::output_db_path(
            output_dir.as_ref().to_str().unwrap(),
            "unused",
            DataBuild::version(),
        );

        Self {
            source_control_content_provider,
            data_content_provider,
            output_db_addr,
            compiler_options,
            registry: None,
        }
    }

    /// Create new instance of `DataBuildOptions` with the mandatory options.
    pub fn new(
        output_db_addr: String,
        source_control_content_provider: Arc<Provider>,
        data_content_provider: Arc<Provider>,
        compiler_options: CompilerRegistryOptions,
    ) -> Self {
        Self {
            source_control_content_provider,
            data_content_provider,
            output_db_addr,
            compiler_options,
            registry: None,
        }
    }

    const OUTPUT_NAME_PREFIX: &'static str = "build_output-";

    /// Construct output database path from:
    /// * a mysql:// path
    /// * an absolute directory or a directory relative to `cwd`.
    ///
    /// The function return `path` if it already contains database name in it.
    pub fn output_db_path(path: &str, cwd: impl AsRef<Path>, version: &str) -> String {
        if path.contains(Self::OUTPUT_NAME_PREFIX) {
            return path.to_owned();
        }

        if path.starts_with("mysql://") {
            let mut output = path.to_owned();
            output.push_str(Self::OUTPUT_NAME_PREFIX);
            output.push_str(version);
            output
        } else {
            Self::output_db_path_dir(PathBuf::from(path), cwd, version)
        }
    }

    /// Construct output database path from an absolute directory or directory relative to `cwd`.
    pub fn output_db_path_dir(
        path: impl AsRef<Path>,
        cwd: impl AsRef<Path>,
        version: &str,
    ) -> String {
        let mut output = "sqlite://".to_string();
        let path = if path.as_ref().is_absolute() {
            path.as_ref().to_owned()
        } else {
            cwd.as_ref().join(path)
        };
        output.push_str(
            &path
                .join(Self::OUTPUT_NAME_PREFIX)
                .to_str()
                .unwrap()
                .replace('\\', "/"),
        );
        output.push_str(version);
        output.push_str(".db3");
        output
    }

    /// Set asset registry used by data compilers. If it is not set `DataBuild` will use
    /// a new instance of asset registry.
    #[must_use]
    pub fn asset_registry(mut self, registry: Arc<AssetRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Set the build output database path.
    /// `build_output_db_addr` can be:
    /// * myslq:// path
    /// * absolute directory path or directory path relative to `cwd`
    #[must_use]
    pub fn output_database(
        mut self,
        build_output_db_addr: &str,
        cwd: impl AsRef<Path>,
        version: &str,
    ) -> Self {
        self.output_db_addr = Self::output_db_path(build_output_db_addr, cwd, version);
        self
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
