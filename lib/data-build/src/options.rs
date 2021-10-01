use std::path::{Path, PathBuf};

use legion_content_store::ContentStoreAddr;

use crate::{DataBuild, Error};

/// Options and flags used by [`DataBuild`].
///
/// To open or create `DataBuild` first call [`DataBuildOptions::new`], then chain calls to
/// methods to set different options, then call [`DataBuildOptions::open_or_create`].
/// This will give  you a [`Result`] with a [`DataBuild`] that you can further operate on.
///
/// # Example Usage
///
/// ```
/// # use legion_data_build::DataBuildOptions;
/// # use legion_content_store::ContentStoreAddr;
/// let mut build = DataBuildOptions::new("./build.index")
///         .content_store(&ContentStoreAddr::from("./content_store/"))
///         .compiler_dir("./compilers/")
///         .create(".");
/// ```
#[derive(Clone, Debug)]
pub struct DataBuildOptions {
    pub(crate) buildindex_path: PathBuf,
    pub(crate) contentstore_path: ContentStoreAddr,
    pub(crate) compiler_search_paths: Vec<PathBuf>,
}

impl DataBuildOptions {
    /// Creates a new data build options.
    pub fn new(buildindex_path: impl AsRef<Path>) -> Self {
        Self {
            buildindex_path: buildindex_path.as_ref().to_owned(),
            contentstore_path: ContentStoreAddr::from(buildindex_path.as_ref()),
            compiler_search_paths: vec![],
        }
    }

    /// Set content store location for derived resources.
    pub fn content_store(&mut self, contentstore_path: &ContentStoreAddr) -> &mut Self {
        self.contentstore_path = contentstore_path.clone();
        self
    }

    /// Adds a directory to compiler search paths.
    pub fn compiler_dir<T: AsRef<Path>>(&mut self, dir: T) -> &mut Self {
        self.compiler_search_paths.push(dir.as_ref().to_owned());
        self
    }

    /// Opens the existing build index.
    ///
    /// If the build index does not exist it creates one if a project is present in the same directory.
    pub fn open_or_create(&self, project_dir: impl AsRef<Path>) -> Result<DataBuild, Error> {
        DataBuild::open_or_create(self, project_dir.as_ref())
    }

    /// Opens existing build index.
    ///
    /// The following conditions need to be met to successfully open a build index:
    /// * [`ContentStore`](`legion_content_store::ContentStore`) must exist under address set by [`DataBuildOptions::content_store()`].
    /// * Build index must exist and be of a supported version provided by [`DataBuildOptions::new()`].
    /// * The build index must point to an existing [`legion_data_offline::resource::Project`].
    pub fn open(&self) -> Result<DataBuild, Error> {
        DataBuild::open(self)
    }

    /// Create new build index for a specified project.
    pub fn create(&self, project_dir: impl AsRef<Path>) -> Result<DataBuild, Error> {
        DataBuild::new(self, project_dir.as_ref())
    }
}
