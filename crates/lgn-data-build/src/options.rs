use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use lgn_content_store::ContentStoreAddr;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::Project;
use lgn_data_runtime::{manifest::Manifest, AssetRegistry};

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
/// let build = DataBuildOptions::new("temp/".to_string(), ContentStoreAddr::from("./content_store/"), CompilerRegistryOptions::local_compilers("./"))
///         .create(&project).await.unwrap();
/// # })
/// ```
pub struct DataBuildOptions {
    pub(crate) contentstore_addr: ContentStoreAddr,
    pub(crate) output_db_addr: String,
    pub(crate) compiler_options: CompilerRegistryOptions,
    pub(crate) registry: Option<Arc<AssetRegistry>>,
    pub(crate) manifest: Option<Manifest>,
}

impl DataBuildOptions {
    /// Creates a new data build options.
    pub fn new_with_sqlite_output(
        output_dir: impl AsRef<Path>,
        compiler_options: CompilerRegistryOptions,
    ) -> Self {
        assert!(output_dir.as_ref().is_absolute());
        let output_db_addr = Self::output_db_path(
            output_dir.as_ref().to_str().unwrap(),
            "unused",
            DataBuild::version(),
        );

        Self {
            contentstore_addr: ContentStoreAddr::from(output_dir.as_ref()),
            output_db_addr,
            compiler_options,
            registry: None,
            manifest: None,
        }
    }

    /// Create new instance of `DataBuildOptions` with the mandatory options.
    pub fn new(
        output_db_addr: String,
        contentstore_addr: ContentStoreAddr,
        compiler_options: CompilerRegistryOptions,
    ) -> Self {
        Self {
            contentstore_addr,
            output_db_addr,
            compiler_options,
            registry: None,
            manifest: None,
        }
    }

    const OUTPUT_NAME_PREFIX: &'static str = "build_output-";

    /// Construct output database path from:
    /// * a mysql:// path
    /// * an absolute directory or a directory relative to `project_dir`.
    ///
    /// The function return `path` if it already contains database name in it.
    pub fn output_db_path(path: &str, project_dir: impl AsRef<Path>, version: &str) -> String {
        if path.contains(Self::OUTPUT_NAME_PREFIX) {
            return path.to_owned();
        }

        if path.starts_with("mysql://") {
            let mut output = path.to_owned();
            output.push_str(Self::OUTPUT_NAME_PREFIX);
            output.push_str(version);
            output
        } else {
            Self::output_db_path_dir(PathBuf::from(path), project_dir, version)
        }
    }

    /// Construct output database path from an absolute directory or directory relative to `project_dir`.
    pub fn output_db_path_dir(
        path: impl AsRef<Path>,
        project_dir: impl AsRef<Path>,
        version: &str,
    ) -> String {
        let mut output = "sqlite://".to_string();
        let path = if path.as_ref().is_absolute() {
            path.as_ref().to_owned()
        } else {
            project_dir.as_ref().join(path)
        };
        output.push_str(
            &path
                .join(Self::OUTPUT_NAME_PREFIX)
                .to_str()
                .unwrap()
                .replace("\\", "/"),
        );
        output.push_str(version);
        output.push_str(".db3");
        output
    }

    /// Set content store location for derived resources.
    pub fn content_store(mut self, contentstore_path: &ContentStoreAddr) -> Self {
        self.contentstore_addr = contentstore_path.clone();
        self
    }

    /// Set asset registry used by data compilers. If it is not set `DataBuild` will use
    /// a new instance of asset registry.
    pub fn asset_registry(mut self, registry: Arc<AssetRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Set manifest used by the asset registry during data compilation.
    pub fn manifest(mut self, manifest: Manifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    /// Set the build output database path.
    /// `build_output_path` can be:
    /// * myslq:// path
    /// * absolute directory path or directory path relative to `project_dir`
    pub fn output_database(
        mut self,
        build_output_path: &str,
        project_dir: impl AsRef<Path>,
        version: &str,
    ) -> Self {
        self.output_db_addr = Self::output_db_path(build_output_path, project_dir, version);
        self
    }

    /// Create new build index for a specified project.
    ///
    /// `project_dir` must be either an absolute path or path relative to
    /// `buildindex_dir`.
    pub async fn create_with_project(
        self,
        project_dir: impl AsRef<Path>,
    ) -> Result<(DataBuild, Project), Error> {
        let project = Project::open(project_dir).await.map_err(Error::from)?;
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
    pub async fn open(self, project: &Project) -> Result<DataBuild, Error> {
        DataBuild::open(self, project).await
    }

    /// Create new build index for a specified project.
    pub async fn create(self, project: &Project) -> Result<DataBuild, Error> {
        DataBuild::new(self, project).await
    }
}
