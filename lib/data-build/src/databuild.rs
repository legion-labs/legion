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
use legion_data_compiler::{CompiledResource, Manifest};
use legion_data_compiler::{Locale, Platform, Target};
use legion_data_offline::{resource::Project, ResourcePathId};
use legion_data_runtime::ResourceType;
use petgraph::algo;

use crate::asset_file_writer::write_assetfile;
use crate::buildindex::{BuildIndex, CompiledResourceInfo, CompiledResourceReference};
use crate::{DataBuildOptions, Error};

#[derive(Clone, Debug)]
struct CompileStat {
    time: std::time::Duration,
    from_cache: bool,
}

struct CompileOutput {
    resources: Vec<CompiledResourceInfo>,
    references: Vec<CompiledResourceReference>,
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
    transform: (ResourceType, ResourceType),
    compiler_hash: CompilerHash,
    databuild_version: &'static str,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    transform.hash(&mut hasher);
    compiler_hash.hash(&mut hasher);
    databuild_version.hash(&mut hasher);
    hasher.finish()
}

/// Data build interface.
///
/// `DataBuild` provides methods to compile offline resources into runtime format.
///
/// Data build uses file-based storage to persist the state of data builds and data compilation.
/// It requires access to offline resources to retrieve resource metadata - through  [`legion_data_offline::resource::Project`].
///
/// # Example Usage
///
/// ```no_run
/// # use legion_data_build::{DataBuild, DataBuildOptions};
/// # use legion_content_store::ContentStoreAddr;
/// # use legion_data_compiler::{Locale, Platform, Target};
/// # use legion_data_offline::ResourcePathId;
/// # use legion_data_runtime::{ResourceId, ResourceType};
/// # use std::str::FromStr;
/// # let offline_anim = ResourceId::from_str("invalid").unwrap();
/// # const RUNTIME_ANIM: ResourceType = ResourceType::new(b"invalid");
/// let mut build = DataBuildOptions::new("./build.index")
///         .content_store(&ContentStoreAddr::from("./content_store/"))
///         .compiler_dir("./compilers/")
///         .create(".").expect("new build index");
///
/// build.source_pull().expect("successful source pull");
/// let manifest_file = &DataBuild::default_output_file();
/// let compile_path = ResourcePathId::from(offline_anim).push(RUNTIME_ANIM);
///
/// let manifest = build.compile(
///                         compile_path,
///                         &manifest_file,
///                         Target::Game,
///                         Platform::Windows,
///                         &Locale::new("en"),
///                      ).expect("compilation output");
/// ```
pub struct DataBuild {
    build_index: BuildIndex,
    project: Project,
    content_store: HddContentStore,
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

        let content_store = HddContentStore::open(config.assetstore_path.clone())
            .ok_or(Error::InvalidAssetStore)?;

        Ok(Self {
            build_index,
            project,
            content_store,
            config: config.clone(),
        })
    }

    pub(crate) fn open(config: &DataBuildOptions) -> Result<Self, Error> {
        let content_store = HddContentStore::open(config.assetstore_path.clone())
            .ok_or(Error::InvalidAssetStore)?;

        let build_index = BuildIndex::open(&config.buildindex_path, Self::version())?;
        let project = build_index.open_project()?;
        Ok(Self {
            build_index,
            project,
            content_store,
            config: config.clone(),
        })
    }

    /// Opens the existing build index.
    ///
    /// If the build index does not exist it creates one if a project is present in the directory.
    pub(crate) fn open_or_create(
        config: &DataBuildOptions,
        project_dir: &Path,
    ) -> Result<Self, Error> {
        let content_store = HddContentStore::open(config.assetstore_path.clone())
            .ok_or(Error::InvalidAssetStore)?;
        match BuildIndex::open(&config.buildindex_path, Self::version()) {
            Ok(build_index) => {
                let project = build_index.open_project()?;
                Ok(Self {
                    build_index,
                    project,
                    content_store,
                    config: config.clone(),
                })
            }
            Err(Error::NotFound) => Self::new(config, project_dir),
            Err(e) => Err(e),
        }
    }

    fn open_project(project_dir: &Path) -> Result<Project, Error> {
        Project::open(project_dir).map_err(|e| match e {
            legion_data_offline::resource::Error::ParseError => Error::IntegrityFailure,
            legion_data_offline::resource::Error::NotFound
            | legion_data_offline::resource::Error::InvalidPath => Error::NotFound,
            legion_data_offline::resource::Error::IOError(_) => Error::IOError,
        })
    }

    /// Accessor for the project associated with this builder.
    pub fn project(&self) -> &Project {
        &self.project
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

    /// Compile `compile_path` resource and all its dependencies in the build graph.
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
        compile_path: ResourcePathId,
        manifest_file: &Path,
        target: Target,
        platform: Platform,
        locale: &Locale,
    ) -> Result<Manifest, Error> {
        let source = compile_path.source_resource();
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
                if file.metadata().unwrap().len() != 0 {
                    let manifest_content: Manifest =
                        serde_json::from_reader(&file).map_err(|_e| Error::InvalidManifest)?;
                    (manifest_content, file)
                } else {
                    (Manifest::default(), file)
                }
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
            resources,
            references,
            statistics: _stats,
        } = self.compile_path(compile_path, target, platform, locale)?;

        let assets = self.link(&resources, &references)?;

        for asset in assets {
            if let Some(existing) = manifest
                .compiled_resources
                .iter_mut()
                .find(|existing| existing.path == asset.path)
            {
                *existing = asset;
            } else {
                manifest.compiled_resources.push(asset);
            }
        }

        file.set_len(0).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        manifest.pre_serialize();
        serde_json::to_writer_pretty(&file, &manifest).map_err(|_e| Error::InvalidManifest)?;

        Ok(manifest)
    }

    /// Compile `compile_node` of the build graph and update *build index* one or more compilation results.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    fn compile_node(
        &mut self,
        compile_node: &ResourcePathId,
        context_hash: u64,
        source_hash: u64,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        target: Target,
        platform: Platform,
        locale: &Locale,
        compiler_path: &Path,
    ) -> Result<
        (
            Vec<CompiledResourceInfo>,
            Vec<CompiledResourceReference>,
            Vec<CompileStat>,
        ),
        Error,
    > {
        let (resource_infos, resource_references, stats): (
            Vec<CompiledResourceInfo>,
            Vec<CompiledResourceReference>,
            _,
        ) = {
            let now = SystemTime::now();
            if let Some((cached_infos, cached_references)) =
                self.build_index
                    .find_compiled(compile_node, context_hash, source_hash)
            {
                let resource_count = cached_infos.len();
                (
                    cached_infos,
                    cached_references,
                    std::iter::repeat(CompileStat {
                        time: now.elapsed().unwrap(),
                        from_cache: true,
                    })
                    .take(resource_count)
                    .collect::<Vec<_>>(),
                )
            } else {
                let mut compile_cmd = CompilerCompileCmd::new(
                    compile_node,
                    dependencies,
                    derived_deps,
                    &self.content_store.address(),
                    &self.project.resource_dir(),
                    target,
                    platform,
                    locale,
                );

                let CompilerCompileCmdOutput {
                    compiled_resources,
                    resource_references,
                } = compile_cmd
                    .execute(compiler_path)
                    .map_err(Error::CompilerError)?;

                self.build_index.insert_compiled(
                    compile_node,
                    context_hash,
                    source_hash,
                    &compiled_resources,
                    &resource_references,
                );
                let resource_count = compiled_resources.len();
                (
                    compiled_resources
                        .iter()
                        .map(|resource| CompiledResourceInfo {
                            context_hash: context_hash.into(),
                            compile_path: compile_node.clone(),
                            source_hash: source_hash.into(),
                            compiled_path: resource.path.clone(),
                            compiled_checksum: resource.checksum,
                            compiled_size: resource.size,
                        })
                        .collect(),
                    resource_references
                        .iter()
                        .map(|reference| CompiledResourceReference {
                            context_hash: context_hash.into(),
                            compile_path: compile_node.clone(),
                            source_hash: source_hash.into(),
                            compiled_path: reference.0.clone(),
                            compiled_reference: reference.1.clone(),
                        })
                        .collect(),
                    std::iter::repeat(CompileStat {
                        time: now.elapsed().unwrap(),
                        from_cache: false,
                    })
                    .take(resource_count)
                    .collect::<Vec<_>>(),
                )
            }
        };

        Ok((resource_infos, resource_references, stats))
    }

    /// Compile a resource identified by [`ResourcePathId`] and all its dependencies and update the *build index* with compilation results.
    /// Returns a list of (id, checksum, size) of created resources and information about their dependencies.
    /// The returned results can be accessed by  [`legion_content_store::ContentStore`] specified in [`DataBuildOptions`] used to create this `DataBuild`.
    // TODO: The list might contain many versions of the same [`ResourceId`] compiled for many contexts (platform, target, locale, etc).
    fn compile_path(
        &mut self,
        compile_path: ResourcePathId,
        target: Target,
        platform: Platform,
        locale: &Locale,
    ) -> Result<CompileOutput, Error> {
        let build_graph = self.build_index.generate_build_graph(compile_path);

        let topological_order: Vec<_> =
            algo::toposort(&build_graph, None).map_err(|_e| Error::CircularDependency)?;

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
                for node in &topological_order {
                    let path = build_graph.node_weight(*node).unwrap();
                    if path.is_source() {
                        continue;
                    }

                    if let Some(transform) = path.last_transform() {
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
                        .find(|info| info.1.transform == transform)
                        .map_or(Err(Error::CompilerNotFound), |e| {
                            let res = compiler_hash_cmd
                                .execute(&e.0.path)
                                .map_err(Error::CompilerError)?;

                            Ok((transform, (e.0.path.clone(), res.compiler_hash)))
                        })
                })
                .collect::<Result<HashMap<_, _>, _>>()?
        };
        let mut compiled_resources = vec![];
        let mut compiled_references = vec![];
        let mut compile_stats = vec![];

        //
        // for now, each node's compilation output contributes to `derived dependencies`
        // as a whole. consecutive nodes will have all derived outputs available.
        //
        // in the future this should be improved.
        //
        let mut accumulated_dependencies = vec![];
        let mut node_hash = HashMap::<_, (u64, u64)>::new();

        for compile_node_index in topological_order {
            let compile_node = build_graph.node_weight(compile_node_index).unwrap();
            // compile non-source dependencies.
            if let Some(direct_dependency) = compile_node.direct_dependency() {
                let mut n = build_graph.neighbors_directed(compile_node_index, petgraph::Incoming);
                let direct_dependency_index = n.next().unwrap();

                // only one direct dependency supported now. it's ok for the path
                // but it needs to be revisited for source (if this ever applies to source).
                assert!(n.next().is_none());

                assert_eq!(
                    &direct_dependency,
                    build_graph.node_weight(direct_dependency_index).unwrap()
                );

                let transform = compile_node.last_transform().unwrap();

                //  'name' is dropped as we always compile input as a whole.
                let compile_node = compile_node.to_unnamed();

                //
                // for derived resources the build index will not have dependencies for.
                // for now derived resources do not inherit dependencies from resources down the
                // resource path.
                //
                // this is compensated by the fact that the compilation output of each node
                // contributes to `derived dependencies`. we might still want to inherit the
                // regular dependencies.
                //
                // this has to be reevaluated in the future.
                //
                let dependencies = self
                    .build_index
                    .find_dependencies(&direct_dependency)
                    .unwrap_or_default();

                let (compiler_path, compiler_hash) = compiler_details.get(&transform).unwrap();

                // todo: not sure if transform is the right thing here. resource_path_id better? transform is already defined by the compiler_hash so it seems redundant.
                let context_hash = compute_context_hash(transform, *compiler_hash, Self::version());

                let source_hash = {
                    if direct_dependency.is_source() {
                        //
                        // todo(kstasik): source_hash computation can include filtering of resource types in the future.
                        // the same resource can have a different source_hash depending on the compiler
                        // used as compilers can filter dependencies out.
                        //
                        self.build_index
                            .compute_source_hash(compile_node.clone())?
                            .get()
                    } else {
                        //
                        // since this is a path-derived resource its hash is equal to the
                        // checksum of its direct dependency.
                        // this is because the direct dependency is the only dependency.
                        // more thought needs to be put into this - this would mean this
                        // resource should not read any other resources - but right now
                        // `accumulated_dependencies` allows to read much more.
                        //
                        let (dep_context_hash, dep_source_hash) =
                            node_hash.get(&direct_dependency_index).unwrap();

                        // we can assume there are results of compilation of the `direct_dependency`
                        let compiled = self
                            .build_index
                            .find_compiled(
                                &direct_dependency.to_unnamed(),
                                *dep_context_hash,
                                *dep_source_hash,
                            )
                            .unwrap()
                            .0;
                        // can we assume there is a result of a requested name?
                        // probably no, this should return a compile error.
                        let source = compiled
                            .iter()
                            .find(|&compiled| compiled.compiled_path == direct_dependency)
                            .unwrap();

                        // this is how we truncate the 128 bit long checksum
                        // and convert it to a 64 bit source_hash.
                        let mut hasher = DefaultHasher::new();
                        source.compiled_checksum.hash(&mut hasher);
                        hasher.finish()
                    }
                };

                node_hash.insert(compile_node_index, (context_hash, source_hash));

                let (resource_infos, resource_references, stats) = self.compile_node(
                    &compile_node,
                    context_hash,
                    source_hash,
                    &dependencies,
                    &accumulated_dependencies,
                    target,
                    platform,
                    locale,
                    compiler_path,
                )?;

                accumulated_dependencies.extend(resource_infos.iter().map(|res| {
                    CompiledResource {
                        path: res.compiled_path.clone(),
                        checksum: res.compiled_checksum,
                        size: res.compiled_size,
                    }
                }));

                compiled_resources.extend(resource_infos);
                compile_stats.extend(stats);
                compiled_references.extend(resource_references);
            }
        }
        Ok(CompileOutput {
            resources: compiled_resources,
            references: compiled_references,
            statistics: compile_stats,
        })
    }

    /// Create asset files in runtime format containing compiled resources that include reference (load-time dependency) information
    /// based on provided compilation information.
    /// Currently each resource is linked into a separate *asset file*.
    fn link(
        &mut self,
        resources: &[CompiledResourceInfo],
        references: &[CompiledResourceReference],
    ) -> Result<Vec<CompiledResource>, Error> {
        let mut resource_files = Vec::with_capacity(resources.len());
        for resource in resources {
            //
            // for now, every derived resource gets an `assetfile` representation.
            //
            let asset_id = resource.compiled_path.content_id();

            let mut output: Vec<u8> = vec![];
            let resource_list = std::iter::once((asset_id, resource.compiled_checksum.get()));
            let reference_list = references
                .iter()
                .filter(|r| r.is_reference_of(resource))
                .map(|r| {
                    (
                        resource.compiled_path.content_id(),
                        (
                            r.compiled_reference.content_id(),
                            r.compiled_reference.content_id(),
                        ),
                    )
                });

            let bytes_written = write_assetfile(
                resource_list,
                reference_list,
                &self.content_store,
                &mut output,
            )?;

            let checksum = self
                .content_store
                .store(&output)
                .ok_or(Error::InvalidAssetStore)?;

            let asset_file = CompiledResource {
                path: resource.compiled_path.clone(),
                checksum: checksum.into(),
                size: bytes_written,
            };
            resource_files.push(asset_file);
        }

        Ok(resource_files)
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

// todo(kstasik): file IO on destructor - is it ok?
impl Drop for DataBuild {
    fn drop(&mut self) {
        self.build_index.flush().unwrap();
    }
}

#[cfg(test)]
#[path = "test_general.rs"]
mod test_general;

#[cfg(test)]
#[path = "test_source_pull.rs"]
mod test_source_pull;

#[cfg(test)]
#[path = "test_compile.rs"]
mod test_compile;
