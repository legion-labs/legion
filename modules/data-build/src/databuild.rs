use std::collections::hash_map::DefaultHasher;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::Seek;
use std::path::{Path, PathBuf};
use std::{env, io};

use legion_data_compiler::compiled_asset_store::{CompiledAssetStoreAddr, LocalCompiledAssetStore};
use legion_data_compiler::compiler_cmd::{
    list_compilers, CompilerCompileCmd, CompilerHashCmd, CompilerInfo, CompilerInfoCmd,
    CompilerInfoCmdOutput,
};
use legion_data_compiler::{CompiledAsset, CompilerHash, Manifest};
use legion_data_compiler::{Locale, Platform, Target};
use legion_resources::{Project, ResourceId, ResourcePathRef, ResourceType};

use crate::buildindex::BuildIndex;
use crate::Error;

const DATABUILD_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Context hash represents all that goes into resource compilation
/// excluding the resource itself.
///
/// The resource itself is represented by `source_hash`.
/// Data compilation of the tuple (`context_hash`, `source_hash`) will always
/// yield the same compilation outcome.
// todo(kstasik): `context_hash` should also include localization_id
fn compute_context_hash(
    resource_type: ResourceType,
    compiler_hash: CompilerHash,
    databuild_version: &'static str,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    resource_type.hash(&mut hasher);
    compiler_hash.hash(&mut hasher);
    databuild_version.hash(&mut hasher);
    hasher.finish()
}

/// Options and flags used by [`DataBuild`].
///
/// To open or create `DataBuild` first call [`DataBuildOptions::new`], then chain calls to
/// methods to set different options, then call [`DataBuildOptions::open_or_create`].
/// This will give  you a [`Result`] with a [`DataBuild`] that you can further operate on.
#[derive(Clone)]
pub struct DataBuildOptions {
    buildindex_path: PathBuf,
    assetstore_path: CompiledAssetStoreAddr,
    compiler_search_paths: Vec<PathBuf>,
}

impl DataBuildOptions {
    /// Creates a new data build options.
    pub fn new(buildindex_path: impl AsRef<Path>) -> Self {
        Self {
            buildindex_path: buildindex_path.as_ref().to_owned(),
            assetstore_path: CompiledAssetStoreAddr::from(buildindex_path.as_ref()),
            compiler_search_paths: vec![],
        }
    }

    /// Set asset store location for compiled assets.
    pub fn asset_store(&mut self, assetstore_path: &CompiledAssetStoreAddr) -> &mut Self {
        self.assetstore_path = assetstore_path.clone();
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
    pub fn open_or_create(&self) -> Result<DataBuild, Error> {
        DataBuild::open_or_create(self)
    }

    /// Opens existing build index.
    ///
    /// The following conditions need to be met to successfully open a build index:
    /// * [`CompiledAssetStore`](`legion_data_compiler::compiled_asset_store::CompiledAssetStore`) must exist under address set by [`DataBuildOptions::asset_store()`].
    /// * Build index must exist and be of a supported version provided by [`DataBuildOptions::new()`].
    /// * The build index must point to an existing [`Project`].
    pub fn open(&self) -> Result<DataBuild, Error> {
        DataBuild::open(self)
    }

    /// Create new build index for a specified project.
    pub fn create(&self, project_dir: impl AsRef<Path>) -> Result<DataBuild, Error> {
        DataBuild::new(self, project_dir.as_ref())
    }
}

/// Data build interface.
///
/// `DataBuild` provides methods to compile offline resources into runtime format.
///
/// Data build uses file-based storage to persist the state of data builds and data compilation.
/// It requires access to offline resources to retrieve resource metadata - throught  [`legion_resources::Project`].
pub struct DataBuild {
    build_index: BuildIndex,
    project: Project,
    asset_store: LocalCompiledAssetStore,
    config: DataBuildOptions,
}

impl DataBuild {
    fn new(config: &DataBuildOptions, project_dir: &Path) -> Result<Self, Error> {
        let project = Self::open_project(project_dir)?;

        let build_index = BuildIndex::create_new(
            &config.buildindex_path,
            &project.indexfile_path(),
            Self::version(),
        )
        .map_err(|_e| Error::IOError)?;

        let asset_store =
            LocalCompiledAssetStore::open(config.assetstore_path.clone()).ok_or(Error::NotFound)?;

        Ok(Self {
            build_index,
            project,
            asset_store,
            config: config.clone(),
        })
    }

    fn open(config: &DataBuildOptions) -> Result<Self, Error> {
        let asset_store =
            LocalCompiledAssetStore::open(config.assetstore_path.clone()).ok_or(Error::NotFound)?;

        let build_index = BuildIndex::open(&config.buildindex_path, Self::version())?;
        let project = build_index.open_project()?;
        Ok(Self {
            build_index,
            project,
            asset_store,
            config: config.clone(),
        })
    }

    /// Opens the existing build index.
    ///
    /// If the build index does not exist it creates one if a project is present in the directory.
    fn open_or_create(config: &DataBuildOptions) -> Result<Self, Error> {
        // todo(kstasik): better error
        let asset_store =
            LocalCompiledAssetStore::open(config.assetstore_path.clone()).ok_or(Error::NotFound)?;
        match BuildIndex::open(&config.buildindex_path, Self::version()) {
            Ok(build_index) => {
                let project = build_index.open_project()?;
                Ok(Self {
                    build_index,
                    project,
                    asset_store,
                    config: config.clone(),
                })
            }
            Err(Error::NotFound) => {
                let projectindex_path = config.buildindex_path.clone(); // we are going to try to locate the project index in the same directory
                Self::new(config, &projectindex_path)
            }
            Err(e) => Err(e),
        }
    }

    fn map_resource_reference(
        id: ResourceId,
        references: &[ResourceId],
    ) -> Result<ResourceId, Error> {
        if let Some(p) = references.iter().find(|&e| *e == id) {
            return Ok(*p);
        }
        Err(Error::IntegrityFailure)
    }

    fn open_project(project_dir: &Path) -> Result<Project, Error> {
        Project::open(project_dir).map_err(|e| match e {
            legion_resources::Error::ParseError => Error::IntegrityFailure,
            legion_resources::Error::NotFound | legion_resources::Error::InvalidPath => {
                Error::NotFound
            }
            legion_resources::Error::IOError(_) => Error::IOError,
        })
    }

    /// Updates the build database with information about resources from provided resource database.
    pub fn source_pull(&mut self) -> Result<i32, Error> {
        let mut updated_resources = 0;

        let all_resources = self.project.resource_list();

        for res in &all_resources {
            let (resource_hash, deps) = self.project.collect_resource_info(*res)?;
            let dependencies = deps
                .into_iter()
                .map(|d| Self::map_resource_reference(d, &all_resources))
                .collect::<Result<Vec<ResourceId>, Error>>()?;

            if self
                .build_index
                .update_resource(*res, resource_hash, dependencies)
            {
                updated_resources += 1;
            }
        }

        Ok(updated_resources)
    }

    // compile_input:
    // - compiler_hash: (asset_type, databuild_ver, compiler_id, loc_id)
    // - source_guid: guid of source resource
    // - source_hash: asset_hash (checksum of meta, checksum of content, flags) + asset_hash(es) of deps
    // compile_output:
    // - compiled_guid
    // - compiled_type
    // - compiled_checksum
    // - compiled_size
    // - compiled_flags

    /// Compiles a named resource and all its dependencies. The compilation results are stored in `compilation database`.
    ///
    /// The data compilation results in a `manifest` that describes the resulting runtime resources.
    pub fn compile(
        &mut self,
        root_resource_name: &ResourcePathRef,
        manifest_file: &Path,
        target: Target,
        platform: Platform,
        locale: &Locale,
    ) -> Result<Manifest, Error> {
        let resource_id = self.project.find_resource(root_resource_name)?;

        // todo(kstasik): for now dependencies are not compiled - only the root resource is.
        let (source_guid, dependencies) =
            self.build_index.find(resource_id).ok_or(Error::NotFound)?;

        let compilers = list_compilers(&self.config.compiler_search_paths);

        let info_cmd = CompilerInfoCmd::default();
        let compilers: Vec<(CompilerInfo, CompilerInfoCmdOutput)> = compilers
            .iter()
            .filter_map(|info| {
                info_cmd
                    .execute(&info.path)
                    .ok()
                    .map(|res| ((*info).clone(), res))
            })
            .collect();

        // todo: compare data_build/rustc version.

        let (compiler_file, _) = compilers
            .iter()
            .find(|info| info.1.resource_type.contains(&source_guid.resource_type()))
            .ok_or(Error::CompilerNotFound)?;

        let compiler_path = &compiler_file.path;

        // todo(kstasik): support triggering compilation for multiple platforms

        let compiler_hash_cmd = CompilerHashCmd::new(target, platform, locale);
        let compiler_hash = compiler_hash_cmd
            .execute(compiler_path)
            .map_err(Error::CompilerError)?;

        assert_eq!(compiler_hash.compiler_hash_list.len(), 1); // todo: support more.
        let compiler_hash = compiler_hash.compiler_hash_list[0];
        let context_hash =
            compute_context_hash(source_guid.resource_type(), compiler_hash, Self::version());

        //
        // todo(kstasik): source_hash computation can include filtering of resource types in the future.
        // the same resource can have a different source_hash depending on the compiler
        // used as compilers can filter dependencies out.
        //
        let source_hash = self.build_index.compute_source_hash(source_guid)?;

        let compiled_assets = {
            let cached = self.build_index.find_compiled(context_hash, source_hash);
            if !cached.is_empty() {
                cached
                    .iter()
                    .map(|asset| CompiledAsset {
                        guid: asset.compiled_guid,
                        checksum: asset.compiled_checksum,
                        size: asset.compiled_size,
                    })
                    .collect()
            } else {
                // for now we only focus on top level asset
                // todo(kstasik): how do we know that GI needs to be run? taking many assets as arguments?

                let mut compile_cmd = CompilerCompileCmd::new(
                    source_guid,
                    dependencies,
                    &self.asset_store.address(),
                    &self.project.resource_dir(),
                    target,
                    platform,
                    locale,
                );

                // todo: what is the cwd for if we provide resource_dir() ?
                let compiled_assets = compile_cmd
                    .execute(compiler_path, &self.project.resource_dir())
                    .map_err(Error::CompilerError)?
                    .compiled_assets;

                self.build_index.insert_compiled(
                    context_hash,
                    source_guid,
                    source_hash,
                    &compiled_assets,
                );
                compiled_assets
            }
        };

        let (mut manifest, mut file) = {
            if let Ok(file) = OpenOptions::new()
                .read(true)
                .write(true)
                .append(false)
                .open(manifest_file)
            {
                let manifest_content: Manifest =
                    serde_json::from_reader(&file).map_err(|_e| Error::InvalidManifest)?;
                (manifest_content, file)
            } else {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(manifest_file)
                    .map_err(|_e| Error::InvalidManifest)?;

                (Manifest::default(), file)
            }
        };

        for asset in compiled_assets {
            if let Some(existing) = manifest
                .compiled_assets
                .iter_mut()
                .find(|existing| existing.guid == asset.guid)
            {
                *existing = asset;
            } else {
                manifest.compiled_assets.push(asset);
            }
        }

        file.set_len(0).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&file, &manifest).map_err(|_e| Error::InvalidManifest)?;

        Ok(manifest)
    }

    /// Returns the global version of the databuild module.
    pub fn version() -> &'static str {
        DATABUILD_VERSION
    }

    /// The default name of the output .manifest file.
    pub fn default_output_file() -> PathBuf {
        PathBuf::from("output.manifest")
    }

    /// Returns the path to the output .manifest file for given build name.
    pub fn manifest_output_file(build_name: &str) -> Result<PathBuf, io::Error> {
        Ok(env::current_dir()?
            .join(build_name)
            .with_extension("manifest"))
    }
}

// todo(kstasik): file IO on descructor - is it ok?
impl Drop for DataBuild {
    fn drop(&mut self) {
        self.build_index.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {

    use std::env;
    use std::fs::{self, File};
    use std::path::PathBuf;

    use crate::{buildindex::BuildIndex, databuild::DataBuild, DataBuildOptions};
    use legion_data_compiler::compiled_asset_store::{
        CompiledAssetStore, CompiledAssetStoreAddr, LocalCompiledAssetStore,
    };
    use legion_data_compiler::{Locale, Manifest, Platform, Target};
    use legion_resources::{test_resource, Project, ResourcePath, ResourceRegistry, ResourceType};

    pub const TEST_BUILDINDEX_FILENAME: &str = "build.index";

    const RESOURCE_MATERIAL: ResourceType = ResourceType::new(b"material");

    fn setup_registry() -> ResourceRegistry {
        let mut resources = ResourceRegistry::default();
        resources.register_type(
            test_resource::TYPE_ID,
            Box::new(test_resource::TestResourceProc {}),
        );
        resources.register_type(
            RESOURCE_MATERIAL,
            Box::new(test_resource::TestResourceProc {}),
        );
        resources
    }

    #[test]
    fn create() {
        let work_dir = tempfile::tempdir().unwrap();
        let project_dir = work_dir.path();
        let projectindex_path = {
            let project = Project::create_new(project_dir).expect("failed to create a project");
            project.indexfile_path()
        };
        let cas_addr = CompiledAssetStoreAddr::from(work_dir.path().to_owned());

        let buildindex_path = project_dir.join(TEST_BUILDINDEX_FILENAME);

        {
            let _build = DataBuildOptions::new(&buildindex_path)
                .asset_store(&cas_addr)
                .create(project_dir)
                .expect("valid data build index");
        }

        let index = BuildIndex::open(&buildindex_path, DataBuild::version())
            .expect("failed to open build index file");

        assert!(index.validate_project_index());

        fs::remove_file(projectindex_path).unwrap();

        assert!(!index.validate_project_index());
    }

    #[test]
    fn source_pull() {
        let work_dir = tempfile::tempdir().unwrap();
        let project_dir = work_dir.path();

        let mut resources = setup_registry();

        {
            let mut project = Project::create_new(project_dir).expect("failed to create a project");

            let texture = project
                .add_resource(
                    ResourcePath::from("child"),
                    test_resource::TYPE_ID,
                    &resources.new_resource(test_resource::TYPE_ID).unwrap(),
                    &mut resources,
                )
                .unwrap();

            let resource = {
                let res = resources.new_resource(test_resource::TYPE_ID).unwrap();
                res.get_mut::<test_resource::TestResource>(&mut resources)
                    .unwrap()
                    .build_deps
                    .push(texture);
                res
            };
            let _material = project
                .add_resource(
                    ResourcePath::from("parent"),
                    RESOURCE_MATERIAL,
                    &resource,
                    &mut resources,
                )
                .unwrap();
        }

        let mut config = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME));
        config.asset_store(&CompiledAssetStoreAddr::from(project_dir.to_owned()));

        {
            let mut build = config.create(project_dir).expect("to create index");

            let updated_count = build.source_pull().unwrap();
            assert_eq!(updated_count, 2);

            let updated_count = build.source_pull().unwrap();
            assert_eq!(updated_count, 0);
        }

        {
            let mut project = Project::open(project_dir).unwrap();
            project
                .add_resource(
                    ResourcePath::from("orphan"),
                    test_resource::TYPE_ID,
                    &resources.new_resource(test_resource::TYPE_ID).unwrap(),
                    &mut resources,
                )
                .unwrap();
        }

        {
            let mut build = config.open().expect("to open index");
            let updated_count = build.source_pull().unwrap();
            assert_eq!(updated_count, 1);
        }
    }

    fn target_dir() -> PathBuf {
        env::current_exe().ok().map_or_else(
            || panic!("cannot find test directory"),
            |mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            },
        )
    }

    #[test]
    fn compile() {
        let work_dir = tempfile::tempdir().unwrap();
        let project_dir = work_dir.path();
        let mut resources = setup_registry();
        {
            let mut project = Project::create_new(project_dir).expect("failed to create a project");

            let texture = project
                .add_resource(
                    ResourcePath::from("child"),
                    test_resource::TYPE_ID,
                    &resources.new_resource(test_resource::TYPE_ID).unwrap(),
                    &mut resources,
                )
                .unwrap();

            let material_handle = resources.new_resource(RESOURCE_MATERIAL).unwrap();
            material_handle
                .get_mut::<test_resource::TestResource>(&mut resources)
                .unwrap()
                .build_deps
                .push(texture);

            let _material = project
                .add_resource(
                    ResourcePath::from("parent"),
                    RESOURCE_MATERIAL,
                    &material_handle,
                    &mut resources,
                )
                .unwrap();
        }

        let assetstore_path = CompiledAssetStoreAddr::from(work_dir.path());
        let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
            .asset_store(&assetstore_path)
            .compiler_dir(target_dir())
            .create(project_dir)
            .expect("to create index");

        build.source_pull().unwrap();

        let output_manifest_file = work_dir.path().join(&DataBuild::default_output_file());

        let manifest = build
            .compile(
                &ResourcePath::from("child"),
                &output_manifest_file,
                Target::Game,
                Platform::Windows,
                &Locale::new("en"),
            )
            .unwrap();

        assert_eq!(manifest.compiled_assets.len(), 1); // for now only the root asset is compiled

        let compiled_checksum = manifest.compiled_assets[0].checksum;
        let asset_store = LocalCompiledAssetStore::open(assetstore_path).unwrap();
        assert!(asset_store.exists(compiled_checksum));

        assert!(output_manifest_file.exists());
        let read_manifest: Manifest = {
            let manifest_file = File::open(&output_manifest_file).unwrap();
            serde_json::from_reader(&manifest_file).unwrap()
        };

        assert_eq!(
            read_manifest.compiled_assets.len(),
            manifest.compiled_assets.len()
        );

        build
            .compile(
                &ResourcePath::from("child"),
                &output_manifest_file,
                Target::Game,
                Platform::Windows,
                &Locale::new("en"),
            )
            .unwrap();

        assert!(output_manifest_file.exists());
        let read_manifest: Manifest = {
            let manifest_file = File::open(&output_manifest_file).unwrap();
            serde_json::from_reader(&manifest_file).unwrap()
        };

        assert_eq!(
            read_manifest.compiled_assets.len(),
            manifest.compiled_assets.len()
        );
    }

    #[test]
    fn resource_modify_compile() {
        let work_dir = tempfile::tempdir().unwrap();
        let project_dir = work_dir.path();
        let mut resources = setup_registry();

        let (resource_id, resource_handle) = {
            let mut project = Project::create_new(project_dir).expect("failed to create a project");

            let resource_handle = resources.new_resource(test_resource::TYPE_ID).unwrap();
            let resource_id = project
                .add_resource(
                    ResourcePath::from("child"),
                    test_resource::TYPE_ID,
                    &resource_handle,
                    &mut resources,
                )
                .unwrap();
            (resource_id, resource_handle)
        };

        let assetstore_path = CompiledAssetStoreAddr::from(work_dir.path());
        let mut config = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME));
        config
            .asset_store(&assetstore_path)
            .compiler_dir(target_dir());

        let output_manifest_file = work_dir.path().join(&DataBuild::default_output_file());

        let original_checksum = {
            let mut build = config.create(project_dir).expect("to create index");
            build.source_pull().expect("failed to pull from project");

            let manifest = build
                .compile(
                    &ResourcePath::from("child"),
                    &output_manifest_file,
                    Target::Game,
                    Platform::Windows,
                    &Locale::new("en"),
                )
                .unwrap();

            assert_eq!(manifest.compiled_assets.len(), 1);

            let original_checksum = manifest.compiled_assets[0].checksum;

            {
                let asset_store = LocalCompiledAssetStore::open(assetstore_path.clone()).unwrap();
                assert!(asset_store.exists(original_checksum));
            }

            assert!(output_manifest_file.exists());
            let read_manifest: Manifest = {
                let manifest_file = File::open(&output_manifest_file).unwrap();
                serde_json::from_reader(&manifest_file).unwrap()
            };

            assert_eq!(
                read_manifest.compiled_assets.len(),
                manifest.compiled_assets.len()
            );

            assert_eq!(read_manifest.compiled_assets[0].checksum, original_checksum);
            original_checksum
        };

        let mut project = Project::open(project_dir).expect("failed to open project");

        resource_handle
            .get_mut::<test_resource::TestResource>(&mut resources)
            .unwrap()
            .content = String::from("new content");

        project
            .save_resource(resource_id, &resource_handle, &mut resources)
            .unwrap();

        let modified_checksum = {
            let mut build = config.open().expect("to open index");
            build.source_pull().expect("failed to pull from project");
            let manifest = build
                .compile(
                    &ResourcePath::from("child"),
                    &output_manifest_file,
                    Target::Game,
                    Platform::Windows,
                    &Locale::new("en"),
                )
                .unwrap();

            assert_eq!(manifest.compiled_assets.len(), 1);

            let modified_checksum = manifest.compiled_assets[0].checksum;

            {
                let asset_store = LocalCompiledAssetStore::open(assetstore_path).unwrap();
                assert!(asset_store.exists(original_checksum));
                assert!(asset_store.exists(modified_checksum));
            }

            assert!(output_manifest_file.exists());
            let read_manifest: Manifest = {
                let manifest_file = File::open(&output_manifest_file).unwrap();
                serde_json::from_reader(&manifest_file).unwrap()
            };

            assert_eq!(
                read_manifest.compiled_assets.len(),
                manifest.compiled_assets.len()
            );

            assert_eq!(read_manifest.compiled_assets[0].checksum, modified_checksum);
            modified_checksum
        };

        assert_ne!(original_checksum, modified_checksum);
    }
}
