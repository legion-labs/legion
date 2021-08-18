use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::Seek;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{env, io};

use legion_content_store::{ContentStore, HddContentStore};
use legion_data_compiler::compiler_api::DATA_BUILD_VERSION;
use legion_data_compiler::compiler_cmd::{
    list_compilers, CompilerCompileCmd, CompilerCompileCmdOutput, CompilerHashCmd, CompilerInfo,
    CompilerInfoCmd, CompilerInfoCmdOutput,
};
use legion_data_compiler::CompilerHash;
use legion_data_compiler::{CompiledAsset, Manifest};
use legion_data_compiler::{Locale, Platform, Target};
use legion_resources::{Project, ResourcePathId, ResourceType};

use crate::asset_file_writer::write_assetfile;
use crate::buildindex::{BuildIndex, CompiledAssetInfo, CompiledAssetReference};
use crate::{DataBuildOptions, Error};

#[derive(Clone, Debug)]
struct CompileStat {
    time: std::time::Duration,
    from_cache: bool,
}

struct CompileOutput {
    asset_objects: Vec<CompiledAssetInfo>,
    references: Vec<CompiledAssetReference>,
    statistics: Vec<CompileStat>,
}

/// Context hash represents all that goes into resource compilation
/// excluding the resource itself.
///
/// The resource itself is represented by `source_hash`.
/// Data compilation of the tuple (`context_hash`, `source_hash`) will always
/// yield the same compilation outcome.
// todo(kstasik): `context_hash` should also include localization_id
fn compute_context_hash(
    resource_type: (ResourceType, ResourceType),
    compiler_hash: CompilerHash,
    databuild_version: &'static str,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    resource_type.hash(&mut hasher);
    compiler_hash.hash(&mut hasher);
    databuild_version.hash(&mut hasher);
    hasher.finish()
}

/// Data build interface.
///
/// `DataBuild` provides methods to compile offline resources into runtime format.
///
/// Data build uses file-based storage to persist the state of data builds and data compilation.
/// It requires access to offline resources to retrieve resource metadata - throught  [`legion_resources::Project`].
///
/// # Example Usage
///
/// ```no_run
/// # use legion_data_build::{DataBuild, DataBuildOptions};
/// # use legion_content_store::ContentStoreAddr;
/// # use legion_data_compiler::{Locale, Platform, Target};
/// # use legion_resources::{ResourceId, ResourcePathId, ResourceType};
/// # use std::str::FromStr;
/// # let offline_anim = ResourceId::from_str("invalid").unwrap();
/// # const RUNTIME_ANIM: ResourceType = ResourceType::new(b"invalid");
/// let mut build = DataBuildOptions::new("./build.index")
///         .asset_store(&ContentStoreAddr::from("./asset_store/"))
///         .compiler_dir("./compilers/")
///         .create(".").expect("new build index");
///
/// build.source_pull().expect("successful source pull");
/// let manifest_file = &DataBuild::default_output_file();
/// let derived = ResourcePathId::from(offline_anim).transform(RUNTIME_ANIM);
///
/// let manifest = build.compile(
///                         derived,
///                         &manifest_file,
///                         Target::Game,
///                         Platform::Windows,
///                         &Locale::new("en"),
///                      ).expect("compilation output");
/// ```
pub struct DataBuild {
    build_index: BuildIndex,
    project: Project,
    asset_store: HddContentStore,
    config: DataBuildOptions,
}

impl DataBuild {
    pub(crate) fn new(config: &DataBuildOptions, project_dir: &Path) -> Result<Self, Error> {
        let project = Self::open_project(project_dir)?;

        let build_index = BuildIndex::create_new(
            &config.buildindex_path,
            &project.indexfile_path(),
            Self::version(),
        )
        .map_err(|_e| Error::IOError)?;

        let asset_store = HddContentStore::open(config.assetstore_path.clone())
            .ok_or(Error::InvalidAssetStore)?;

        Ok(Self {
            build_index,
            project,
            asset_store,
            config: config.clone(),
        })
    }

    pub(crate) fn open(config: &DataBuildOptions) -> Result<Self, Error> {
        let asset_store = HddContentStore::open(config.assetstore_path.clone())
            .ok_or(Error::InvalidAssetStore)?;

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
    pub(crate) fn open_or_create(config: &DataBuildOptions) -> Result<Self, Error> {
        let asset_store = HddContentStore::open(config.assetstore_path.clone())
            .ok_or(Error::InvalidAssetStore)?;
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

        for resource_id in &all_resources {
            let (resource_hash, resource_deps) = self.project.resource_info(*resource_id)?;

            if self.build_index.update_resource(
                ResourcePathId::from(*resource_id),
                Some(resource_hash),
                resource_deps.clone(),
            ) {
                updated_resources += 1;
            }

            // add each derived dependency with it's direct dependency listed in deps.
            for dependency in resource_deps {
                if let Some(direct_dependency) = dependency.direct_dependency() {
                    if self
                        .build_index
                        .update_resource(dependency, None, vec![direct_dependency])
                    {
                        updated_resources += 1;
                    }
                }
            }
        }

        Ok(updated_resources)
    }

    /// Compiles a resource at `derived` node in compilation graph.
    ///
    /// To compile a given `ResourcePathId` it compiles all its dependent derived resources.
    /// The specified `manifest_file` is updated with information about changed assets.
    ///
    /// Compilation results are stored in [`ContentStore`](`legion_content_store::ContentStore`)
    /// specified in [`DataBuildOptions`] used to create this `DataBuild`.
    ///
    /// Provided `target`, `platform` and `locale` define the compilation context that can yield different compilation results.
    pub fn compile(
        &mut self,
        derived: ResourcePathId,
        manifest_file: &Path,
        target: Target,
        platform: Platform,
        locale: &Locale,
    ) -> Result<Manifest, Error> {
        let source = derived.source_resource();
        if !self.project.exists(source) {
            return Err(Error::NotFound);
        }

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

        let CompileOutput {
            asset_objects,
            references,
            statistics: _stats,
        } = self.compile_path(derived, target, platform, locale)?;

        let assets = self.link(&asset_objects, &references)?;

        for asset in assets {
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

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    fn compile_node(
        &mut self,
        derived: &ResourcePathId,
        context_hash: u64,
        source_hash: u64,
        dependencies: &[ResourcePathId],
        target: Target,
        platform: Platform,
        locale: &Locale,
        compiler_path: &Path,
    ) -> Result<
        (
            Vec<CompiledAssetInfo>,
            Vec<CompiledAssetReference>,
            Vec<CompileStat>,
        ),
        Error,
    > {
        let (asset_objects, asset_object_references, stats): (
            Vec<CompiledAssetInfo>,
            Vec<CompiledAssetReference>,
            _,
        ) = {
            let now = SystemTime::now();
            let (cached_asset_objects, cached_references) =
                self.build_index
                    .find_compiled(derived, context_hash, source_hash);
            if !cached_asset_objects.is_empty() {
                let asset_count = cached_asset_objects.len();
                (
                    cached_asset_objects,
                    cached_references,
                    std::iter::repeat(CompileStat {
                        time: now.elapsed().unwrap(),
                        from_cache: true,
                    })
                    .take(asset_count)
                    .collect::<Vec<_>>(),
                )
            } else {
                let mut compile_cmd = CompilerCompileCmd::new(
                    derived,
                    dependencies,
                    &self.asset_store.address(),
                    &self.project.resource_dir(),
                    target,
                    platform,
                    locale,
                );

                // todo: what is the cwd for if we provide resource_dir() ?
                let CompilerCompileCmdOutput {
                    compiled_assets,
                    asset_references,
                } = compile_cmd
                    .execute(compiler_path, &self.project.resource_dir())
                    .map_err(Error::CompilerError)?;

                self.build_index.insert_compiled(
                    context_hash,
                    derived,
                    source_hash,
                    &compiled_assets,
                    &asset_references,
                );
                let asset_count = compiled_assets.len();
                (
                    compiled_assets
                        .iter()
                        .map(|asset| CompiledAssetInfo {
                            context_hash,
                            source_guid: derived.clone(),
                            source_hash,
                            compiled_guid: asset.guid,
                            compiled_checksum: asset.checksum,
                            compiled_size: asset.size,
                        })
                        .collect(),
                    asset_references
                        .iter()
                        .map(|reference| CompiledAssetReference {
                            context_hash,
                            source_guid: derived.clone(),
                            source_hash,
                            compiled_guid: reference.0,
                            compiled_reference: reference.1,
                        })
                        .collect(),
                    std::iter::repeat(CompileStat {
                        time: now.elapsed().unwrap(),
                        from_cache: false,
                    })
                    .take(asset_count)
                    .collect::<Vec<_>>(),
                )
            }
        };

        Ok((asset_objects, asset_object_references, stats))
    }

    /// Compiles a resource by [`ResourcePathId`]. Returns a list of ids of `Asset Objects` compiled.
    /// The list might contain many versions of the same [`AssetId`] compiled for many contexts (platform, target, locale, etc).
    /// Those results are in [`ContentStore`](`legion_data_compiler::ContentStore`)
    /// specified in [`DataBuildOptions`] used to create this `DataBuild`.
    fn compile_path(
        &mut self,
        derived: ResourcePathId,
        target: Target,
        platform: Platform,
        locale: &Locale,
    ) -> Result<CompileOutput, Error> {
        // todo: rename this: `compile order`?
        let ordered_nodes = self.build_index.evaluation_order(derived)?;

        let compiler_details = {
            let compilers = list_compilers(&self.config.compiler_search_paths);

            let info_cmd = CompilerInfoCmd::default();
            let compilers: Vec<(CompilerInfo, CompilerInfoCmdOutput)> = compilers
                .iter()
                .filter_map(|info| {
                    info_cmd
                        .execute(&info.path)
                        .ok()
                        .filter(|res| res.build_version == Self::version())
                        .map(|res| ((*info).clone(), res))
                })
                .collect();

            let unique_transforms = {
                let mut transforms = vec![];
                for node in &ordered_nodes {
                    if node.is_source() {
                        continue;
                    }

                    if let Some(transform) = node.last_transform() {
                        transforms.push(transform);
                    }
                }
                transforms.sort();
                transforms.dedup();
                transforms
            };

            let compiler_hash_cmd = CompilerHashCmd::new(target, platform, locale);

            unique_transforms
                .into_iter()
                .map(|transform| {
                    compilers
                        .iter()
                        .find(|info| info.1.transforms.contains(&transform))
                        .map_or(Err(Error::CompilerNotFound), |e| {
                            let res = compiler_hash_cmd
                                .execute(&e.0.path)
                                .map_err(Error::CompilerError)?;

                            Ok((transform, (e.0.path.clone(), res.compiler_hash_list)))
                        })
                })
                .collect::<Result<HashMap<_, _>, _>>()?
        };
        let mut compiled_assets = vec![];
        let mut compiled_references = vec![];
        let mut compile_stats = vec![];

        for output_id in ordered_nodes {
            // compile non-source dependencies.
            if let Some(input_id) = output_id.direct_dependency() {
                let transform = output_id.last_transform().unwrap();
                let dependencies = self
                    .build_index
                    .find_dependencies(&input_id)
                    .ok_or(Error::NotFound)?;

                let (compiler_path, compiler_hash_list) = compiler_details.get(&transform).unwrap();

                // todo(kstasik): support triggering compilation for multiple platforms

                assert_eq!(compiler_hash_list.len(), 1); // todo: support more.
                let compiler_hash = compiler_hash_list[0];

                // todo: not sure if transofrm is the right thing here. resource_path_id better? transform is already defined by the compiler_hash so it seems redundant.
                let context_hash = compute_context_hash(transform, compiler_hash, Self::version());

                //
                // todo(kstasik): source_hash computation can include filtering of resource types in the future.
                // the same resource can have a different source_hash depending on the compiler
                // used as compilers can filter dependencies out.
                //
                // todo: is `output_id` correct here? of `input_id` should be used? (seems correct)
                let source_hash = self.build_index.compute_source_hash(output_id.clone())?;

                let (asset_objects, asset_object_references, stats) = self.compile_node(
                    &output_id,
                    context_hash,
                    source_hash,
                    &dependencies,
                    target,
                    platform,
                    locale,
                    compiler_path,
                )?;

                compiled_assets.extend(asset_objects);
                compile_stats.extend(stats);
                compiled_references.extend(asset_object_references);
            }
        }
        Ok(CompileOutput {
            asset_objects: compiled_assets,
            references: compiled_references,
            statistics: compile_stats,
        })
    }

    fn link(
        &mut self,
        asset_objects: &[CompiledAssetInfo],
        references: &[CompiledAssetReference],
    ) -> Result<Vec<CompiledAsset>, Error> {
        let mut asset_files = Vec::with_capacity(asset_objects.len());
        for asset_object in asset_objects {
            let mut output: Vec<u8> = vec![];
            let asset_list =
                std::iter::once((asset_object.compiled_guid, asset_object.compiled_checksum));
            let reference_list = references
                .iter()
                .filter(|r| r.is_reference_of(asset_object))
                .map(|r| {
                    (
                        asset_object.compiled_guid,
                        (r.compiled_reference, r.compiled_reference),
                    )
                });
            let bytes_written =
                write_assetfile(asset_list, reference_list, &self.asset_store, &mut output)?;

            let checksum = self
                .asset_store
                .store(&output)
                .ok_or(Error::InvalidAssetStore)?;

            let asset_file = CompiledAsset {
                guid: asset_object.compiled_guid,
                checksum,
                size: bytes_written,
            };
            asset_files.push(asset_file);
        }

        Ok(asset_files)
    }

    /// Returns the global version of the databuild module.
    pub fn version() -> &'static str {
        DATA_BUILD_VERSION
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

    use std::fs::{self, File};
    use std::path::{Path, PathBuf};
    use std::{env, vec};

    use crate::databuild::CompileOutput;
    use crate::{buildindex::BuildIndex, databuild::DataBuild, DataBuildOptions};
    use legion_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
    use legion_data_compiler::{Locale, Manifest, Platform, Target};
    use legion_resources::{Project, ResourceId, ResourceName, ResourcePathId, ResourceRegistry};

    pub const TEST_BUILDINDEX_FILENAME: &str = "build.index";

    fn setup_registry() -> ResourceRegistry {
        let mut resources = ResourceRegistry::default();
        resources.register_type(
            test_resource::TYPE_ID,
            Box::new(test_resource::TestResourceProc {}),
        );
        resources.register_type(
            test_resource::TYPE_ID,
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
        let cas_addr = ContentStoreAddr::from(work_dir.path().to_owned());

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

            let child_id = project
                .add_resource(
                    ResourceName::from("child"),
                    test_resource::TYPE_ID,
                    &resources.new_resource(test_resource::TYPE_ID).unwrap(),
                    &mut resources,
                )
                .unwrap();

            let parent_handle = {
                let res = resources.new_resource(test_resource::TYPE_ID).unwrap();
                res.get_mut::<test_resource::TestResource>(&mut resources)
                    .unwrap()
                    .build_deps
                    .push(ResourcePathId::from(child_id));
                res
            };
            let _parent_id = project
                .add_resource(
                    ResourceName::from("parent"),
                    test_resource::TYPE_ID,
                    &parent_handle,
                    &mut resources,
                )
                .unwrap();
        }

        let mut config = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME));
        config.asset_store(&ContentStoreAddr::from(project_dir.to_owned()));

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
                    ResourceName::from("orphan"),
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

        {
            let mut project = Project::open(project_dir).unwrap();

            let child_id = project
                .add_resource(
                    ResourceName::from("intermediate_child"),
                    test_resource::TYPE_ID,
                    &resources.new_resource(test_resource::TYPE_ID).unwrap(),
                    &mut resources,
                )
                .unwrap();

            let parent_handle = {
                let intermediate_id =
                    ResourcePathId::from(child_id).transform(test_resource::TYPE_ID);

                let res = resources.new_resource(test_resource::TYPE_ID).unwrap();
                res.get_mut::<test_resource::TestResource>(&mut resources)
                    .unwrap()
                    .build_deps
                    .push(intermediate_id);
                res
            };
            let _parent_id = project
                .add_resource(
                    ResourceName::from("intermetidate_parent"),
                    test_resource::TYPE_ID,
                    &parent_handle,
                    &mut resources,
                )
                .unwrap();
        }

        {
            let mut build = config.open().expect("to open index");
            let updated_count = build.source_pull().unwrap();
            assert_eq!(updated_count, 3);

            let updated_count = build.source_pull().unwrap();
            assert_eq!(updated_count, 0);
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
    fn verify_manifest() {
        let work_dir = tempfile::tempdir().unwrap();
        let project_dir = work_dir.path();
        let mut resources = setup_registry();

        // child_id <- test(child_id) <- parent_id = test(parent_id)
        let parent_resource = {
            let mut project = Project::create_new(project_dir).expect("new project");
            let child_id = project
                .add_resource(
                    ResourceName::from("child"),
                    test_resource::TYPE_ID,
                    &resources.new_resource(test_resource::TYPE_ID).unwrap(),
                    &mut resources,
                )
                .unwrap();

            let child_handle = resources.new_resource(test_resource::TYPE_ID).unwrap();
            child_handle
                .get_mut::<test_resource::TestResource>(&mut resources)
                .unwrap()
                .build_deps
                .push(ResourcePathId::from(child_id).transform(test_resource::TYPE_ID));

            project
                .add_resource(
                    ResourceName::from("parent"),
                    test_resource::TYPE_ID,
                    &child_handle,
                    &mut resources,
                )
                .unwrap()
        };

        let assetstore_path = ContentStoreAddr::from(work_dir.path());
        let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
            .asset_store(&assetstore_path)
            .compiler_dir(target_dir())
            .create(project_dir)
            .expect("to create index");

        build.source_pull().unwrap();

        let output_manifest_file = work_dir.path().join(&DataBuild::default_output_file());

        let derived = ResourcePathId::from(parent_resource).transform(test_resource::TYPE_ID);
        let manifest = build
            .compile(
                derived,
                &output_manifest_file,
                Target::Game,
                Platform::Windows,
                &Locale::new("en"),
            )
            .unwrap();

        // both test(child_id) and test(parent_id) are separate assets.
        assert_eq!(manifest.compiled_assets.len(), 2);

        let asset_store = HddContentStore::open(assetstore_path).expect("valid asset store");
        for checksum in manifest.compiled_assets.iter().map(|a| a.checksum) {
            assert!(asset_store.exists(checksum));
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

        for asset in read_manifest.compiled_assets {
            assert!(manifest
                .compiled_assets
                .iter()
                .any(|a| a.checksum == asset.checksum));
        }
    }

    #[test]
    fn compile_change_no_deps() {
        let work_dir = tempfile::tempdir().unwrap();
        let project_dir = work_dir.path();
        let mut resources = setup_registry();

        let (resource_id, resource_handle) = {
            let mut project = Project::create_new(project_dir).expect("failed to create a project");

            let resource_handle = resources.new_resource(test_resource::TYPE_ID).unwrap();
            let resource_id = project
                .add_resource(
                    ResourceName::from("resource"),
                    test_resource::TYPE_ID,
                    &resource_handle,
                    &mut resources,
                )
                .unwrap();
            (resource_id, resource_handle)
        };

        let assetstore_path = ContentStoreAddr::from(work_dir.path());
        let mut config = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME));
        config
            .asset_store(&assetstore_path)
            .compiler_dir(target_dir());

        let target = ResourcePathId::from(resource_id).transform(test_resource::TYPE_ID);

        let original_checksum = {
            let mut build = config.create(project_dir).expect("to create index");
            build.source_pull().expect("failed to pull from project");

            let compile_output = build
                .compile_path(
                    target.clone(),
                    Target::Game,
                    Platform::Windows,
                    &Locale::new("en"),
                )
                .unwrap();

            assert_eq!(compile_output.asset_objects.len(), 1);
            assert_eq!(compile_output.references.len(), 0);

            let original_checksum = compile_output.asset_objects[0].compiled_checksum;

            let asset_store =
                HddContentStore::open(assetstore_path.clone()).expect("valid asset store");
            assert!(asset_store.exists(original_checksum));

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
            let compile_output = build
                .compile_path(target, Target::Game, Platform::Windows, &Locale::new("en"))
                .unwrap();

            assert_eq!(compile_output.asset_objects.len(), 1);

            let modified_checksum = compile_output.asset_objects[0].compiled_checksum;

            let asset_store = HddContentStore::open(assetstore_path).expect("valid asset store");
            assert!(asset_store.exists(original_checksum));
            assert!(asset_store.exists(modified_checksum));

            modified_checksum
        };

        assert_ne!(original_checksum, modified_checksum);
    }

    fn create_resource(
        name: ResourceName,
        deps: &[ResourcePathId],
        project: &mut Project,
        resources: &mut ResourceRegistry,
    ) -> ResourceId {
        let resource_b = {
            let res = resources.new_resource(test_resource::TYPE_ID).unwrap();
            let resource = res
                .get_mut::<test_resource::TestResource>(resources)
                .unwrap();
            resource.content = name.display().to_string(); // each resource needs unique content to generate a unique asset.
            resource.build_deps.extend_from_slice(deps);
            res
        };
        project
            .add_resource(name, test_resource::TYPE_ID, &resource_b, resources)
            .unwrap()
    }

    fn change_resource(resource_id: ResourceId, project_dir: &Path) {
        let mut project = Project::open(project_dir).expect("failed to open project");
        let mut resources = setup_registry();

        let handle = project
            .load_resource(resource_id, &mut resources)
            .expect("to load resource");

        let resource = handle
            .get_mut::<test_resource::TestResource>(&mut resources)
            .expect("resource instance");
        resource.content.push_str(" more content");
        project
            .save_resource(resource_id, &handle, &mut resources)
            .expect("successful save");
    }

    /// Creates a project with 5 resources with dependencies setup as depicted below.
    /// Returns an array of resources from A to E where A is at index 0.
    ///
    /// A -> B -> C
    /// |    |
    /// D -> E
    ///
    fn setup_project(project_dir: impl AsRef<Path>) -> [ResourceId; 5] {
        let mut project =
            Project::create_new(project_dir.as_ref()).expect("failed to create a project");

        let mut resources = setup_registry();

        let res_c = create_resource(ResourceName::from("C"), &[], &mut project, &mut resources);
        let res_e = create_resource(ResourceName::from("E"), &[], &mut project, &mut resources);
        let res_d = create_resource(
            ResourceName::from("D"),
            &[ResourcePathId::from(res_e).transform(test_resource::TYPE_ID)],
            &mut project,
            &mut resources,
        );
        let res_b = create_resource(
            ResourceName::from("B"),
            &[
                ResourcePathId::from(res_c).transform(test_resource::TYPE_ID),
                ResourcePathId::from(res_e).transform(test_resource::TYPE_ID),
            ],
            &mut project,
            &mut resources,
        );
        let res_a = create_resource(
            ResourceName::from("A"),
            &[
                ResourcePathId::from(res_b).transform(test_resource::TYPE_ID),
                ResourcePathId::from(res_d).transform(test_resource::TYPE_ID),
            ],
            &mut project,
            &mut resources,
        );
        [res_a, res_b, res_c, res_d, res_e]
    }

    #[test]
    fn dependency_invalidation() {
        let work_dir = tempfile::tempdir().unwrap();
        let project_dir = work_dir.path();

        let resource_list = setup_project(project_dir);

        let root_resource = resource_list[0];

        let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
            .asset_store(&ContentStoreAddr::from(work_dir.path()))
            .compiler_dir(target_dir())
            .create(project_dir)
            .expect("new build index");
        build.source_pull().expect("successful pull");

        //
        // test(A) -> A -> test(B) -> B -> test(C) -> C
        //            |               |
        //            V               |
        //          test(D)           |
        //            |               |
        //            V               V
        //            D ---------> test(E) -> E
        //
        const NUM_NODES: usize = 10;
        const NUM_OUTPUTS: usize = 5;
        let target = ResourcePathId::from(root_resource).transform(test_resource::TYPE_ID);

        //  test of evaluation order computation.
        {
            let order = build
                .build_index
                .evaluation_order(target.clone())
                .expect("no cycles");
            assert_eq!(order.len(), NUM_NODES);
            assert_eq!(order[NUM_NODES - 1], target);
            assert_eq!(order[NUM_NODES - 2], ResourcePathId::from(root_resource));
        }

        // first run - none of the assets from cache.
        {
            let CompileOutput {
                asset_objects,
                references,
                statistics,
            } = build
                .compile_path(
                    target.clone(),
                    Target::Game,
                    Platform::Windows,
                    &Locale::new("en"),
                )
                .expect("successful compilation");

            assert_eq!(asset_objects.len(), NUM_OUTPUTS);
            assert_eq!(references.len(), NUM_OUTPUTS);
            assert!(statistics.iter().all(|s| !s.from_cache));
        }

        // no change, second run - all assets from cache.
        {
            let CompileOutput {
                asset_objects,
                references,
                statistics,
            } = build
                .compile_path(
                    target.clone(),
                    Target::Game,
                    Platform::Windows,
                    &Locale::new("en"),
                )
                .expect("successful compilation");

            assert_eq!(asset_objects.len(), NUM_OUTPUTS);
            assert_eq!(references.len(), NUM_OUTPUTS);
            assert!(statistics.iter().all(|s| s.from_cache));
        }

        // change root resource, one asset re-compiled.
        {
            change_resource(root_resource, project_dir);
            build.source_pull().expect("to pull changes");

            let CompileOutput {
                asset_objects,
                references,
                statistics,
            } = build
                .compile_path(
                    target.clone(),
                    Target::Game,
                    Platform::Windows,
                    &Locale::new("en"),
                )
                .expect("successful compilation");

            assert_eq!(asset_objects.len(), NUM_OUTPUTS);
            assert_eq!(references.len(), NUM_OUTPUTS);
            assert_eq!(statistics.iter().filter(|s| !s.from_cache).count(), 1);
        }

        // change resource E - which invalides 4 resources in total (E included).
        {
            let resource_e = resource_list[4];
            change_resource(resource_e, project_dir);
            build.source_pull().expect("to pull changes");

            let CompileOutput {
                asset_objects,
                references,
                statistics,
            } = build
                .compile_path(target, Target::Game, Platform::Windows, &Locale::new("en"))
                .expect("successful compilation");

            assert_eq!(asset_objects.len(), 5);
            assert_eq!(references.len(), 5);
            assert_eq!(statistics.iter().filter(|s| !s.from_cache).count(), 4);
        }
    }

    #[test]
    fn link() {
        let work_dir = tempfile::tempdir().unwrap();
        let project_dir = work_dir.path();
        let mut resources = setup_registry();

        let parent_id = {
            let mut project = Project::create_new(project_dir).expect("new project");

            let child_handle = resources
                .new_resource(test_resource::TYPE_ID)
                .expect("valid resource");
            let child = child_handle
                .get_mut::<test_resource::TestResource>(&mut resources)
                .expect("existing resource");
            child.content = String::from("test child content");
            let child_id = project
                .add_resource(
                    ResourceName::from("child"),
                    test_resource::TYPE_ID,
                    &child_handle,
                    &mut resources,
                )
                .unwrap();

            let parent_handle = resources
                .new_resource(test_resource::TYPE_ID)
                .expect("valid resource");
            let parent = parent_handle
                .get_mut::<test_resource::TestResource>(&mut resources)
                .expect("existing resource");
            parent.content = String::from("test parent content");
            parent.build_deps =
                vec![ResourcePathId::from(child_id).transform(test_resource::TYPE_ID)];
            project
                .add_resource(
                    ResourceName::from("parent"),
                    test_resource::TYPE_ID,
                    &parent_handle,
                    &mut resources,
                )
                .unwrap()
        };

        let assetstore_path = ContentStoreAddr::from(work_dir.path());
        let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
            .asset_store(&assetstore_path)
            .compiler_dir(target_dir())
            .create(project_dir)
            .expect("to create index");

        build.source_pull().unwrap();

        // for now each asset is a separate file so we need to validate that the compile output and link output produce the same number of assets

        let target = ResourcePathId::from(parent_id).transform(test_resource::TYPE_ID);
        let compile_output = build
            .compile_path(target, Target::Game, Platform::Windows, &Locale::new("en"))
            .expect("successful compilation");

        assert_eq!(compile_output.asset_objects.len(), 2);
        assert_eq!(compile_output.references.len(), 1);

        let link_output = build
            .link(&compile_output.asset_objects, &compile_output.references)
            .expect("successful linking");

        assert_eq!(compile_output.asset_objects.len(), link_output.len());

        // link output checksum must be different from compile output checksum...
        for obj in &compile_output.asset_objects {
            assert!(!link_output
                .iter()
                .any(|compiled| compiled.checksum == obj.compiled_checksum));
        }

        // ... and each output asset need to exist as exactly one asset object (although having different checksum).
        for output in link_output {
            assert_eq!(
                compile_output
                    .asset_objects
                    .iter()
                    .filter(|obj| obj.compiled_guid == output.guid)
                    .count(),
                1
            );
        }
    }
}
