use std::collections::HashMap;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::Seek;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use std::{env, io};

use lgn_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use lgn_data_compiler::compiler_api::{CompilationEnv, CompilationOutput, DATA_BUILD_VERSION};
use lgn_data_compiler::compiler_node::{CompilerNode, CompilerRegistry, CompilerStub};
use lgn_data_compiler::CompilerHash;
use lgn_data_compiler::{CompiledResource, Manifest};
use lgn_data_offline::Transform;
use lgn_data_offline::{resource::Project, ResourcePathId};
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions, ResourceTypeAndId};
use lgn_utils::{DefaultHash, DefaultHasher};
use petgraph::{algo, Graph};

use crate::asset_file_writer::write_assetfile;
use crate::buildindex::{BuildIndex, CompiledResourceInfo, CompiledResourceReference};
use crate::{DataBuildOptions, Error};

#[derive(Clone, Debug)]
#[allow(dead_code)] // used by tests
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
    transform: Transform,
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
/// `DataBuild` provides methods to compile offline resources into runtime
/// format.
///
/// Data build uses file-based storage to persist the state of data builds and
/// data compilation. It requires access to offline resources to retrieve
/// resource metadata - through  [`lgn_data_offline::resource::Project`].
///
/// # Example Usage
///
/// ```no_run
/// # use lgn_data_build::{DataBuild, DataBuildOptions};
/// # use lgn_content_store::ContentStoreAddr;
/// # use lgn_data_compiler::{compiler_api::CompilationEnv, compiler_node::CompilerRegistryOptions, Locale, Platform, Target};
/// # use lgn_data_offline::ResourcePathId;
/// # use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
/// # use std::str::FromStr;
/// # let offline_anim: ResourceTypeAndId = "(type,invalid_id)".parse::<ResourceTypeAndId>().unwrap();
/// # const RUNTIME_ANIM: ResourceType = ResourceType::new(b"invalid");
/// # tokio_test::block_on(async {
/// let (mut build, project) = DataBuildOptions::new(".", CompilerRegistryOptions::from_dir("./compilers/"))
///         .content_store(&ContentStoreAddr::from("./content_store/"))
///         .create_with_project(".").await.expect("new build index");
///
/// build.source_pull(&project).await.expect("successful source pull");
/// let manifest_file = &DataBuild::default_output_file();
/// let compile_path = ResourcePathId::from(offline_anim).push(RUNTIME_ANIM);
///
/// let env = CompilationEnv {
///            target: Target::Game,
///            platform: Platform::Windows,
///            locale: Locale::new("en"),
/// };
///
/// let manifest = build.compile(
///                         compile_path,
///                         Some(manifest_file.to_path_buf()),
///                         &env,
///                      ).expect("compilation output");
/// # })
/// ```
#[derive(Debug)]
pub struct DataBuild {
    build_index: BuildIndex,
    resource_dir: PathBuf,
    content_store: HddContentStore,
    compilers: CompilerNode,
}

impl DataBuild {
    fn default_asset_registry(
        resource_dir: &Path,
        cas_addr: ContentStoreAddr,
        compilers: &CompilerRegistry,
    ) -> Result<Arc<AssetRegistry>, Error> {
        let source_store = HddContentStore::open(cas_addr).ok_or(Error::InvalidContentStore)?;

        let mut options = AssetRegistryOptions::new()
            .add_device_cas(
                Box::new(source_store),
                lgn_data_runtime::manifest::Manifest::default(),
            )
            .add_device_dir(resource_dir);

        options = compilers.init_all(options);

        Ok(options.create())
    }

    // todo: should not return the Project
    pub(crate) async fn new(
        config: DataBuildOptions,
        project_dir: &Path,
    ) -> Result<(Self, Project), Error> {
        let projectindex_path = Project::root_to_index_path(project_dir);
        let corrected_path =
            BuildIndex::construct_project_path(&config.buildindex_dir, &projectindex_path)?;

        let project = Self::open_project(&corrected_path).await?;

        let build_index =
            BuildIndex::create_new(&config.buildindex_dir, &projectindex_path, Self::version())
                .map_err(|_e| Error::Io)?;

        let content_store = HddContentStore::open(config.contentstore_path.clone())
            .ok_or(Error::InvalidContentStore)?;

        let compilers = config.compiler_options.create();
        let registry = config.registry.map_or_else(
            || {
                Self::default_asset_registry(
                    &project.resource_dir(),
                    config.contentstore_path.clone(),
                    &compilers,
                )
            },
            Ok,
        )?;

        Ok((
            Self {
                build_index,
                resource_dir: project.resource_dir(),
                content_store,
                compilers: CompilerNode::new(compilers, registry),
            },
            project,
        ))
    }

    pub(crate) async fn new_with_proj(
        config: DataBuildOptions,
        project: &Project,
    ) -> Result<Self, Error> {
        let project_dir = project.project_dir();

        let build_index = BuildIndex::create_new(
            &config.buildindex_dir,
            &Project::root_to_index_path(project_dir),
            Self::version(),
        )
        .map_err(|_e| Error::Io)?;

        let content_store = HddContentStore::open(config.contentstore_path.clone())
            .ok_or(Error::InvalidContentStore)?;

        let compilers = config.compiler_options.create();
        let registry = config.registry.map_or_else(
            || {
                Self::default_asset_registry(
                    &project.resource_dir(),
                    config.contentstore_path.clone(),
                    &compilers,
                )
            },
            Ok,
        )?;

        Ok(Self {
            build_index,
            resource_dir: project.resource_dir(),
            content_store,
            compilers: CompilerNode::new(compilers, registry),
        })
    }

    pub(crate) async fn open(config: DataBuildOptions) -> Result<(Self, Project), Error> {
        let build_index = BuildIndex::open(&config.buildindex_dir, Self::version())?;
        let project = build_index.open_project().await?;

        let content_store = HddContentStore::open(config.contentstore_path.clone())
            .ok_or(Error::InvalidContentStore)?;

        let compilers = config.compiler_options.create();
        let registry = config.registry.map_or_else(
            || {
                Self::default_asset_registry(
                    &project.resource_dir(),
                    config.contentstore_path.clone(),
                    &compilers,
                )
            },
            Ok,
        )?;

        Ok((
            Self {
                build_index,
                resource_dir: project.resource_dir(),
                content_store,
                compilers: CompilerNode::new(compilers, registry),
            },
            project,
        ))
    }

    pub(crate) async fn open_with_proj(
        config: DataBuildOptions,
        project: &Project,
    ) -> Result<Self, Error> {
        let build_index = BuildIndex::open(&config.buildindex_dir, Self::version())?;
        // todo: validate project path
        //let project = build_index.open_project().await?;

        let content_store = HddContentStore::open(config.contentstore_path.clone())
            .ok_or(Error::InvalidContentStore)?;

        let compilers = config.compiler_options.create();
        let registry = config.registry.map_or_else(
            || {
                Self::default_asset_registry(
                    &project.resource_dir(),
                    config.contentstore_path.clone(),
                    &compilers,
                )
            },
            Ok,
        )?;

        Ok(Self {
            build_index,
            resource_dir: project.resource_dir(),
            content_store,
            compilers: CompilerNode::new(compilers, registry),
        })
    }

    /// Opens the existing build index.
    ///
    /// If the build index does not exist it creates one if a project is present
    /// in the directory.
    pub(crate) async fn open_or_create(
        config: DataBuildOptions,
        project_dir: &Path,
    ) -> Result<(Self, Project), Error> {
        let content_store = HddContentStore::open(config.contentstore_path.clone())
            .ok_or(Error::InvalidContentStore)?;
        match BuildIndex::open(&config.buildindex_dir, Self::version()) {
            Ok(build_index) => {
                let project = build_index.open_project().await?;

                let compilers = config.compiler_options.create();
                let registry = config.registry.map_or_else(
                    || {
                        Self::default_asset_registry(
                            &project.resource_dir(),
                            config.contentstore_path.clone(),
                            &compilers,
                        )
                    },
                    Ok,
                )?;

                Ok((
                    Self {
                        build_index,
                        resource_dir: project.resource_dir(),
                        content_store,
                        compilers: CompilerNode::new(compilers, registry),
                    },
                    project,
                ))
            }
            Err(Error::NotFound) => Self::new(config, project_dir).await,
            Err(e) => Err(e),
        }
    }

    pub(crate) async fn open_or_create_with_proj(
        config: DataBuildOptions,
        project: &Project,
    ) -> Result<Self, Error> {
        let content_store = HddContentStore::open(config.contentstore_path.clone())
            .ok_or(Error::InvalidContentStore)?;
        match BuildIndex::open(&config.buildindex_dir, Self::version()) {
            Ok(build_index) => {
                // todo: validate the two paths
                //let p = build_index.project_path();
                //let p2 = project.project_dir();

                let compilers = config.compiler_options.create();
                let registry = config.registry.map_or_else(
                    || {
                        Self::default_asset_registry(
                            &project.resource_dir(),
                            config.contentstore_path.clone(),
                            &compilers,
                        )
                    },
                    Ok,
                )?;

                Ok(Self {
                    build_index,
                    resource_dir: project.resource_dir(),
                    content_store,
                    compilers: CompilerNode::new(compilers, registry),
                })
            }
            Err(Error::NotFound) => Self::new_with_proj(config, project).await,
            Err(e) => Err(e),
        }
    }

    async fn open_project(project_dir: &Path) -> Result<Project, Error> {
        Project::open(project_dir).await.map_err(Error::from)
    }

    /// Returns a source of a resource id.
    ///
    /// It will return None if the build never recorded a source for a given id.
    pub fn lookup_pathid(&self, id: ResourceTypeAndId) -> Option<ResourcePathId> {
        self.build_index.output_index.lookup_pathid(id)
    }

    /// Updates the build database with information about resources from
    /// provided resource database.
    pub async fn source_pull(&mut self, project: &Project) -> Result<i32, Error> {
        let mut updated_resources = 0;

        for resource_id in project.resource_list().await {
            let (kind, resource_hash, resource_deps) = project.resource_info(resource_id)?;

            if self.build_index.update_resource(
                ResourcePathId::from(ResourceTypeAndId {
                    id: resource_id,
                    kind,
                }),
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

    /// Compile `compile_path` resource and all its dependencies in the build
    /// graph.
    ///
    /// To compile a given `ResourcePathId` it compiles all its dependent
    /// derived resources. The specified `manifest_file` is updated with
    /// information about changed assets.
    ///
    /// Compilation results are stored in
    /// [`ContentStore`](`lgn_content_store::ContentStore`) specified in
    /// [`DataBuildOptions`] used to create this `DataBuild`.
    ///
    /// Provided `target`, `platform` and `locale` define the compilation
    /// context that can yield different compilation results.
    pub fn compile(
        &mut self,
        compile_path: ResourcePathId,
        manifest_file: Option<PathBuf>,
        env: &CompilationEnv,
    ) -> Result<Manifest, Error> {
        let (mut manifest, file) = {
            if let Some(manifest_file) = manifest_file {
                if let Ok(file) = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .append(false)
                    .open(&manifest_file)
                {
                    if file.metadata().unwrap().len() == 0 {
                        (Manifest::default(), Some(file))
                    } else {
                        let manifest_content: Manifest = serde_json::from_reader(&file)
                            .map_err(|e| Error::InvalidManifest(e.into()))?;
                        (manifest_content, Some(file))
                    }
                } else {
                    let file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create_new(true)
                        .open(manifest_file)
                        .map_err(|e| Error::InvalidManifest(e.into()))?;

                    (Manifest::default(), Some(file))
                }
            } else {
                (Manifest::default(), None)
            }
        };

        let CompileOutput {
            resources,
            references,
            statistics: _stats,
        } = self.compile_path(compile_path, env)?;

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

        if let Some(mut file) = file {
            file.set_len(0).unwrap();
            file.seek(std::io::SeekFrom::Start(0)).unwrap();
            manifest.pre_serialize();
            serde_json::to_writer_pretty(&file, &manifest)
                .map_err(|e| Error::InvalidManifest(e.into()))?;
        }
        Ok(manifest)
    }

    /// Compile `compile_node` of the build graph and update *build index* one
    /// or more compilation results.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    fn compile_node(
        build_index: &mut BuildIndex,
        cas_addr: ContentStoreAddr,
        project_dir: &Path,
        compile_node: &ResourcePathId,
        context_hash: u64,
        source_hash: u64,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        env: &CompilationEnv,
        compiler: &dyn CompilerStub,
        resources: Arc<AssetRegistry>,
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
                build_index
                    .output_index
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
                let CompilationOutput {
                    compiled_resources,
                    resource_references,
                } = compiler
                    .compile(
                        compile_node.clone(),
                        dependencies,
                        derived_deps,
                        resources,
                        cas_addr,
                        project_dir,
                        env,
                    )
                    .map_err(Error::Compiler)?;

                build_index.output_index.insert_compiled(
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

    /// Returns build graph in a Graphviz DOT format.
    ///
    /// Graphviz format documentation can be found [here](https://www.graphviz.org/doc/info/lang.html)
    ///
    /// `std::string::ToString::to_string` can be used as a default
    /// `name_parser`.
    pub fn print_build_graph(
        &self,
        compile_path: ResourcePathId,
        name_parser: impl Fn(&ResourcePathId) -> String,
    ) -> String {
        let build_graph = self
            .build_index
            .source_index
            .generate_build_graph(compile_path);
        #[rustfmt::skip]
        let inner_getter = |_g: &Graph<ResourcePathId, ()>,
                            nr: <&petgraph::Graph<lgn_data_offline::ResourcePathId, ()> as petgraph::visit::IntoNodeReferences>::NodeRef| {
            format!("label = \"{}\"", (name_parser)(nr.1))
        };
        let dot = petgraph::dot::Dot::with_attr_getters(
            &build_graph,
            &[petgraph::dot::Config::EdgeNoLabel],
            &|_, _| String::new(),
            &inner_getter,
        );

        format!("{:?}", dot)
    }

    /// Compile a resource identified by [`ResourcePathId`] and all its
    /// dependencies and update the *build index* with compilation results.
    /// Returns a list of (id, checksum, size) of created resources and
    /// information about their dependencies. The returned results can be
    /// accessed by  [`lgn_content_store::ContentStore`] specified in
    /// [`DataBuildOptions`] used to create this `DataBuild`.
    // TODO: The list might contain many versions of the same [`ResourceId`] compiled for many
    // contexts (platform, target, locale, etc).
    fn compile_path(
        &mut self,
        compile_path: ResourcePathId,
        env: &CompilationEnv,
    ) -> Result<CompileOutput, Error> {
        self.build_index.output_index.record_pathid(&compile_path);

        let build_graph = self
            .build_index
            .source_index
            .generate_build_graph(compile_path);

        let topological_order: Vec<_> = algo::toposort(&build_graph, None).map_err(|_e| {
            eprintln!("{:?}", build_graph);
            Error::CircularDependency
        })?;

        let compiler_details = {
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

            unique_transforms
                .into_iter()
                .map(|transform| {
                    let (compiler, transform) = self
                        .compilers
                        .compilers()
                        .find_compiler(transform)
                        .ok_or(Error::CompilerNotFound)?;
                    let compiler_hash = compiler
                        .compiler_hash(transform, env)
                        .map_err(|_e| Error::Io)?;
                    Ok((transform, (compiler, compiler_hash)))
                })
                .collect::<Result<HashMap<_, _>, Error>>()?
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
                let expected_name = compile_node.name();
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
                    .source_index
                    .find_dependencies(&direct_dependency)
                    .unwrap_or_default();

                let (compiler, compiler_hash) = *compiler_details.get(&transform).unwrap();

                // todo: not sure if transform is the right thing here. resource_path_id better?
                // transform is already defined by the compiler_hash so it seems redundant.
                let context_hash = compute_context_hash(transform, compiler_hash, Self::version());

                let source_hash = {
                    if direct_dependency.is_source() {
                        //
                        // todo(kstasik): source_hash computation can include filtering of resource
                        // types in the future. the same resource can have a
                        // different source_hash depending on the compiler
                        // used as compilers can filter dependencies out.
                        //
                        self.build_index
                            .source_index
                            .compute_source_hash(compile_node.clone())
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
                            .output_index
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
                            .unwrap_or_else(|| {
                                panic!("compilation output of: {}", direct_dependency)
                            });

                        // this is how we truncate the 128 bit long checksum
                        // and convert it to a 64 bit source_hash.
                        source.compiled_checksum.default_hash()
                    }
                };

                node_hash.insert(compile_node_index, (context_hash, source_hash));

                let (resource_infos, resource_references, stats) = Self::compile_node(
                    &mut self.build_index,
                    self.content_store.address(),
                    &self.resource_dir,
                    &compile_node,
                    context_hash,
                    source_hash,
                    &dependencies,
                    &accumulated_dependencies,
                    env,
                    compiler,
                    self.compilers.registry(),
                )?;

                // registry must be updated to release any resources that are no longer referenced.
                self.compilers.registry().update();

                // we check if the expected named output was produced.
                if let Some(expected_name) = expected_name {
                    if !resource_infos.iter().any(|info| {
                        info.compiled_path
                            .name()
                            .map_or(false, |name| name == expected_name)
                    }) {
                        return Err(Error::OutputNotPresent);
                    }
                }

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

    /// Create asset files in runtime format containing compiled resources that
    /// include reference (load-time dependency) information
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
            let asset_id = resource.compiled_path.resource_id();

            let resource_list = std::iter::once((asset_id, resource.compiled_checksum));
            let reference_list = references
                .iter()
                .filter(|r| r.is_reference_of(resource))
                .map(|r| {
                    (
                        resource.compiled_path.resource_id(),
                        (
                            r.compiled_reference.resource_id(),
                            r.compiled_reference.resource_id(),
                        ),
                    )
                });

            let output = write_assetfile(resource_list, reference_list, &self.content_store)?;

            let checksum = self
                .content_store
                .store(&output)
                .ok_or(Error::InvalidContentStore)?;

            let asset_file = CompiledResource {
                path: resource.compiled_path.clone(),
                checksum,
                size: output.len(),
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
