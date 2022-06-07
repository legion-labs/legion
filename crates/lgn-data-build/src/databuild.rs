use std::{
    collections::{HashMap, HashSet},
    env,
    hash::{Hash, Hasher},
    io,
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    time::SystemTime,
};

use futures::{
    future::{select_all, try_join_all},
    Future, FutureExt,
};
use lgn_content_store::{
    indexing::{empty_tree_id, ResourceWriter, SharedTreeIdentifier},
    Provider,
};
use lgn_data_compiler::{
    compiler_api::{CompilationEnv, CompilationOutput, CompilerHash, DATA_BUILD_VERSION},
    compiler_node::{CompilerNode, CompilerStub},
    CompiledResource, CompiledResources,
};
use lgn_data_offline::{resource::Project, vfs::AddDeviceCASOffline};
use lgn_data_runtime::{
    AssetRegistry, AssetRegistryOptions, ResourcePathId, ResourceTypeAndId, Transform,
};
use lgn_tracing::{async_span_scope, debug, error, info};
use lgn_utils::{DefaultHash, DefaultHasher};
use petgraph::{algo, graph::NodeIndex, Graph};

use crate::{
    asset_file_writer::write_assetfile,
    output_index::{AssetHash, CompiledResourceInfo, CompiledResourceReference, OutputIndex},
    source_index::SourceIndex,
    DataBuildOptions, Error,
};

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
) -> AssetHash {
    let mut hasher = DefaultHasher::new();
    transform.hash(&mut hasher);
    compiler_hash.hash(&mut hasher);
    databuild_version.hash(&mut hasher);
    AssetHash::from(hasher.finish())
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
/// # use std::sync::Arc;
/// # use lgn_data_build::{DataBuild, DataBuildOptions};
/// # use lgn_content_store::{ContentProvider, ProviderConfig};
/// # use lgn_data_compiler::{compiler_api::CompilationEnv, compiler_node::CompilerRegistryOptions, Locale, Platform, Target};
/// # use lgn_data_runtime::ResourcePathId;
/// # use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
/// # use std::str::FromStr;
/// # let offline_anim: ResourceTypeAndId = "(type,invalid_id)".parse::<ResourceTypeAndId>().unwrap();
/// # const RUNTIME_ANIM: ResourceType = ResourceType::new(b"invalid");
/// # tokio_test::block_on(async {
/// let source_control_content_provider = Arc::new(Box::new(MemoryProvider::new()));
/// let data_content_provider = Arc::new(Box::new(MemoryProvider::new()));
/// let (mut build, project) = DataBuildOptions::new("temp/".to_string(), data_content_provider, CompilerRegistryOptions::local_compilers("./compilers/"))
///         .create_with_project(".", source_control_content_provider).await.expect("new build index");
///
/// build.source_pull(&project).await.expect("successful source pull");
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
///                         &env,
///                      ).await.expect("compilation output");
/// # })
/// ```
//#[derive(Debug)]
pub struct DataBuild {
    source_index: SourceIndex,
    output_index: OutputIndex,
    runtime_manifest_id: SharedTreeIdentifier,
    data_content_provider: Arc<Provider>,
    compilers: CompilerNode,
}

impl DataBuild {
    pub(crate) async fn new(config: DataBuildOptions, project: &Project) -> Result<Self, Error> {
        let output_index = OutputIndex::create_new(config.output_db_addr.clone()).await?;

        Self::new_with_output_index(config, output_index, project).await
    }

    pub(crate) async fn open(config: DataBuildOptions, project: &Project) -> Result<Self, Error> {
        let output_index = OutputIndex::open(config.output_db_addr.clone()).await?;

        Self::new_with_output_index(config, output_index, project).await
    }

    pub(crate) async fn open_or_create(
        config: DataBuildOptions,
        project: &Project,
    ) -> Result<Self, Error> {
        let output_index = match OutputIndex::open(config.output_db_addr.clone()).await {
            Ok(output_index) => Ok(output_index),
            Err(Error::NotFound(_)) => OutputIndex::create_new(config.output_db_addr.clone()).await,
            Err(e) => Err(e),
        }?;

        Self::new_with_output_index(config, output_index, project).await
    }

    async fn new_with_output_index(
        config: DataBuildOptions,
        output_index: OutputIndex,
        project: &Project,
    ) -> Result<Self, Error> {
        let source_index = SourceIndex::new(Arc::clone(&config.data_content_provider));
        let compilers = config.compiler_options.create().await;
        let runtime_manifest_id =
            SharedTreeIdentifier::new(empty_tree_id(&config.data_content_provider).await.unwrap());

        let registry = match config.registry {
            Some(r) => r,
            None => {
                // setup default asset registry
                let data_provider = Arc::clone(&config.data_content_provider);
                let empty_manifest_id =
                    SharedTreeIdentifier::new(empty_tree_id(&data_provider).await.unwrap());

                let mut options = AssetRegistryOptions::new()
                    .add_device_cas(data_provider, empty_manifest_id)
                    .add_device_cas_offline(
                        Arc::clone(&config.source_control_content_provider),
                        project.offline_manifest_id(),
                    );

                options = compilers.init_all(options).await;

                options.create().await
            }
        };

        Ok(Self {
            source_index,
            output_index,
            runtime_manifest_id,
            data_content_provider: Arc::clone(&config.data_content_provider),
            compilers: CompilerNode::new(compilers, registry),
        })
    }

    /// Returns a source of a resource id.
    ///
    /// It will return None if the build never recorded a source for a given id.
    pub async fn lookup_pathid(
        &self,
        id: ResourceTypeAndId,
    ) -> Result<Option<ResourcePathId>, Error> {
        if let Some(source_index) = self.source_index.current() {
            if let Some(id) = source_index.lookup_pathid(id) {
                return Ok(Some(id));
            }
        }
        self.output_index.lookup_pathid(id).await
    }

    /// Updates the build database with information about resources from
    /// provided resource database.
    pub async fn source_pull(&mut self, project: &Project) -> Result<(), Error> {
        self.source_index
            .source_pull(project, Self::version())
            .await
    }

    /// Compile `compile_path` resource and all its dependencies in the build
    /// graph.
    ///
    /// To compile a given `ResourcePathId` it compiles all its dependent
    /// derived resources. The specified `manifest_file` is updated with
    /// information about changed assets.
    ///
    /// Compilation results are stored in content store specified in
    /// [`DataBuildOptions`] used to create this `DataBuild`.
    ///
    /// Provided `target`, `platform` and `locale` define the compilation
    /// context that can yield different compilation results.
    pub async fn compile(
        &mut self,
        compile_path: ResourcePathId,
        env: &CompilationEnv,
    ) -> Result<CompiledResources, Error> {
        self.output_index.record_pathid(&compile_path).await?;
        let mut result = CompiledResources::default();

        let start = std::time::Instant::now();
        info!("Compilation of {} Started", compile_path);

        let CompileOutput {
            resources,
            references,
            statistics: _stats,
        } = self.compile_path(compile_path, env).await?;

        let assets = self.link(&resources, &references).await?;

        for asset in assets {
            if let Some(existing) = result
                .compiled_resources
                .iter_mut()
                .find(|existing| existing.path == asset.path)
            {
                *existing = asset;
            } else {
                result.compiled_resources.push(asset);
            }
        }

        info!("Compilation Ended ({:?})", start.elapsed());
        Ok(result)
    }

    /// Compile `compile_node` of the build graph and update *build index* one
    /// or more compilation results.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    async fn compile_node(
        output_index: &OutputIndex,
        data_provider: &Provider,
        runtime_manifest_id: &SharedTreeIdentifier,
        compile_node: &ResourcePathId,
        context_hash: AssetHash,
        source_hash: AssetHash,
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
            if let Some((cached_infos, cached_references)) = output_index
                .find_compiled(compile_node, context_hash, source_hash)
                .await
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
                        data_provider,
                        runtime_manifest_id,
                        env,
                    )
                    .await
                    .map_err(Error::Compiler)?;

                // a resource cannot refer to itself
                assert_eq!(
                    resource_references.iter().filter(|(a, b)| a == b).count(),
                    0
                );

                output_index
                    .insert_compiled(
                        compile_node,
                        context_hash,
                        source_hash,
                        &compiled_resources,
                        &resource_references,
                    )
                    .await?;
                let resource_count = compiled_resources.len();
                (
                    compiled_resources
                        .iter()
                        .map(|resource| CompiledResourceInfo {
                            context_hash,
                            compile_path: compile_node.clone(),
                            source_hash,
                            compiled_path: resource.path.clone(),
                            compiled_content_id: resource.content_id.clone(),
                        })
                        .collect(),
                    resource_references
                        .iter()
                        .map(|reference| CompiledResourceReference {
                            context_hash,
                            compile_path: compile_node.clone(),
                            source_hash,
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
    pub async fn print_build_graph<R>(
        &self,
        compile_path: ResourcePathId,
        _name_parser: impl Fn(&ResourcePathId) -> R,
    ) -> String
    where
        R: Future<Output = String>,
    {
        if let Some(source_index) = self.source_index.current() {
            let build_graph = source_index.generate_build_graph(compile_path);
            #[rustfmt::skip]
        let inner_getter = |_g: &Graph<ResourcePathId, ()>,
                            _nr: <&petgraph::Graph<lgn_data_runtime::ResourcePathId, ()> as petgraph::visit::IntoNodeReferences>::NodeRef| {
            format!("label = \"{}\"", "todo"/*(name_parser)(nr.1)*/)
        };
            let dot = petgraph::dot::Dot::with_attr_getters(
                &build_graph,
                &[petgraph::dot::Config::EdgeNoLabel],
                &|_, _| String::new(),
                &inner_getter,
            );

            format!("{:?}", dot)
        } else {
            String::new()
        }
    }

    /// Compile a resource identified by [`ResourcePathId`] and all its
    /// dependencies and update the *build index* with compilation results.
    /// Returns a list of (id, checksum) of created resources and
    /// information about their dependencies. The returned results can be
    /// accessed by  [`ContentStore`] specified in
    /// [`DataBuildOptions`] used to create this `DataBuild`.
    // TODO: The list might contain many versions of the same [`ResourceId`] compiled for many
    // contexts (platform, target, locale, etc).
    async fn compile_path(
        &mut self,
        compile_path: ResourcePathId,
        env: &CompilationEnv,
    ) -> Result<CompileOutput, Error> {
        if self.source_index.current().is_none() {
            return Err(Error::SourceIndex);
        }
        let source_index = self.source_index.current().unwrap();

        let build_graph = source_index.generate_build_graph(compile_path);

        let mut topological_order: Vec<_> = algo::toposort(&build_graph, None).map_err(|_e| {
            eprintln!("{:?}", build_graph);
            Error::CircularDependency
        })?;

        let compiler_details = {
            let unique_transforms: Vec<(Transform, ResourcePathId)> = {
                let mut transforms = vec![];
                for node in &topological_order {
                    let path = build_graph.node_weight(*node).unwrap();
                    if path.is_source() {
                        continue;
                    }

                    if let Some(transform) = path.last_transform() {
                        transforms.push((transform, path.clone()));
                    }
                }
                transforms.sort();
                transforms.dedup();
                transforms
            };

            let mut compiler_details = HashMap::new();
            for t in unique_transforms {
                let (transform, res_path_id) = t;
                let (compiler, transform) = self
                    .compilers
                    .compilers()
                    .find_compiler(transform)
                    .ok_or(Error::CompilerNotFound(transform, res_path_id))?;
                let compiler_hash = compiler
                    .compiler_hash(transform, env)
                    .await
                    .map_err(|e| Error::Io(e.into()))?;
                compiler_details.insert(transform, (compiler, compiler_hash));
            }
            compiler_details
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
        let mut node_hash = HashMap::<_, (AssetHash, AssetHash)>::new();

        let mut compiled_at_node = HashMap::<ResourcePathId, _>::new();
        let mut compiled = HashSet::<petgraph::graph::NodeIndex>::new();

        let mut compiled_unnamed = HashSet::<ResourcePathId>::new();
        let mut compiling_unnamed = HashSet::<ResourcePathId>::new();

        let mut work = vec![];

        let to_compile = topological_order.len();

        for node in &topological_order {
            debug!(
                "Node {:?} -> {}",
                node,
                build_graph.node_weight(*node).unwrap()
            );
        }

        for node in &topological_order {
            debug!(
                "Dependency List {:?}:{}",
                node,
                build_graph
                    .neighbors_directed(*node, petgraph::Incoming)
                    .fold("".to_string(), |acc, d| acc + &format!(" {:?}", d))
            );
        }

        while compiled.len() < to_compile {
            let (ready, pending) =
                topological_order
                    .into_iter()
                    .partition::<Vec<_>, _>(|&compile_node_index| {
                        // ready if all dependencies have been compiled
                        let all_deps_compiled = build_graph
                            .neighbors_directed(compile_node_index, petgraph::Incoming)
                            .all(|index| compiled.contains(&index));

                        let compile_node = build_graph.node_weight(compile_node_index).unwrap();
                        let compile_node_unnamed = compile_node.to_unnamed();

                        // make sure we only schedule one compilation of a node. the rest will be pending until that one completes.
                        {
                            if compiling_unnamed.contains(&compile_node_unnamed) {
                                return false;
                            }
                            if all_deps_compiled
                                && !compiled_unnamed.contains(&compile_node_unnamed)
                            {
                                compiling_unnamed.insert(compile_node_unnamed);
                            }
                        }

                        all_deps_compiled
                    });
            info!(
                "Progress: ready: {}, pending {}, ongoing: {}, done: {}/{}",
                ready.len(),
                pending.len(),
                work.len(),
                compiled.len(),
                to_compile
            );
            topological_order = pending;

            for node in &topological_order {
                let dep_status = build_graph
                    .neighbors_directed(*node, petgraph::Incoming)
                    .fold("".to_string(), |acc, d| {
                        let dd = build_graph.node_weight(d).unwrap();
                        let completion_status = if compiled.contains(&d) {
                            "compiled"
                        } else if compiling_unnamed.contains(&dd.to_unnamed()) {
                            "compiling"
                        } else {
                            "pending"
                        };
                        acc + &format!(" {:?} - {},", d, completion_status)
                    });
                let name = build_graph.node_weight(*node).unwrap().to_unnamed();
                let unnamed_status = if compiling_unnamed.contains(&name) {
                    "compiling"
                } else if compiled_unnamed.contains(&name) {
                    "compiled"
                } else {
                    "pending"
                };
                debug!(
                    "Pending '{:?}': Status: '{}'. Deps:{}",
                    node, unnamed_status, dep_status
                );
            }

            debug!("Compiled: {:?}", compiled);
            debug!("Compiling Unnamed {:?}", compiling_unnamed);
            debug!("Compiled Unnamed {:?}", compiled_unnamed);

            let mut new_work = vec![];
            let num_ready = ready.len();
            for compile_node_index in ready {
                let compile_node = build_graph.node_weight(compile_node_index).unwrap();
                info!(
                    "Progress({:?}): {:?} is ready",
                    compile_node_index, compile_node
                );
                // compile non-source dependencies.
                if let Some(direct_dependency) = compile_node.direct_dependency() {
                    let mut n =
                        build_graph.neighbors_directed(compile_node_index, petgraph::Incoming);
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

                    // check if the unnamed ResourcePathId has been already compiled and early out.
                    if let Some(node_index) = compiled_at_node.get(&compile_node) {
                        node_hash.insert(compile_node_index, *node_hash.get(node_index).unwrap());
                        compiled.insert(compile_node_index);
                        continue;
                    }

                    compiled_at_node.insert(compile_node.clone(), compile_node_index);

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
                    let dependencies = source_index
                        .find_dependencies(&direct_dependency)
                        .unwrap_or_default();

                    let (compiler, compiler_hash) = *compiler_details.get(&transform).unwrap();

                    // todo: not sure if transform is the right thing here. resource_path_id better?
                    // transform is already defined by the compiler_hash so it seems redundant.
                    let context_hash =
                        compute_context_hash(transform, compiler_hash, Self::version());

                    let source_hash = {
                        if direct_dependency.is_source() {
                            //
                            // todo(kstasik): source_hash computation can include filtering of resource
                            // types in the future. the same resource can have a
                            // different source_hash depending on the compiler
                            // used as compilers can filter dependencies out.
                            //
                            source_index.compute_source_hash(compile_node.clone())
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
                                .output_index
                                .find_compiled(
                                    &direct_dependency.to_unnamed(),
                                    *dep_context_hash,
                                    *dep_source_hash,
                                )
                                .await
                                .unwrap()
                                .0;
                            // can we assume there is a result of a requested name?
                            // probably no, this should return a compile error.
                            if let Some(source) = compiled
                                .iter()
                                .find(|&compiled| compiled.compiled_path == direct_dependency)
                            {
                                // this is how we truncate the 128 bit long checksum
                                // and convert it to a 64 bit source_hash.
                                AssetHash::from(source.compiled_content_id.default_hash())
                            } else {
                                lgn_tracing::error!(
                                    "Failed to find compilation output for: {}",
                                    direct_dependency
                                );
                                continue;
                            }
                        }
                    };

                    node_hash.insert(compile_node_index, (context_hash, source_hash));

                    let output_index = &self.output_index;
                    let data_content_provider = &self.data_content_provider;
                    let runtime_manifest_id = &self.runtime_manifest_id;
                    let resources = self.compilers.registry();
                    let acc_deps = accumulated_dependencies.clone();

                    #[allow(clippy::type_complexity)]
                    let work: Pin<
                        Box<dyn Future<Output = Result<_, (NodeIndex, Error)>> + Send>,
                    > = async move {
                        info!(
                            "Compiling({:?}) {} ({:?}) ...",
                            compile_node_index, compile_node, expected_name
                        );
                        let start = std::time::Instant::now();

                        let (resource_infos, resource_references, stats) = Self::compile_node(
                            output_index,
                            data_content_provider,
                            runtime_manifest_id,
                            &compile_node,
                            context_hash,
                            source_hash,
                            &dependencies,
                            &acc_deps,
                            env,
                            compiler,
                            resources.clone(),
                        )
                        .await
                        .map_err(|e| (compile_node_index, e))?;

                        info!(
                            "Compiled({:?}) {:?} ended in {:?}.",
                            compile_node_index,
                            compile_node,
                            start.elapsed()
                        );

                        // registry must be updated to release any resources that are no longer referenced.
                        resources.update();

                        Ok((
                            compile_node_index,
                            resource_infos,
                            resource_references,
                            stats,
                        ))
                    }
                    .boxed();
                    new_work.push(work);
                } else {
                    let unnamed = compile_node.to_unnamed();
                    info!("Source({:?}) Completed '{}'", compile_node_index, unnamed);
                    compiled.insert(compile_node_index);
                    compiling_unnamed.remove(&unnamed);
                    compiled_unnamed.insert(unnamed);
                }
            }

            info!(
                "Progress: new work: {}, total work: {}, pending: {}, done: {}/{}",
                new_work.len(),
                work.len() + new_work.len(),
                topological_order.len(),
                compiled.len(),
                to_compile
            );
            work.extend(new_work);

            if work.is_empty() && num_ready > 0 {
                continue;
            }

            //
            //
            //

            let (result, _, remaining) = select_all(work).await;

            match result {
                Ok((node_index, resource_infos, resource_references, stats)) => {
                    let compile_node = build_graph.node_weight(node_index).unwrap();

                    let unnamed = compile_node.to_unnamed();
                    info!(
                        "Progress({:?}): done: {} ({})",
                        node_index, compile_node, unnamed
                    );
                    compiling_unnamed.remove(&unnamed);
                    compiled_unnamed.insert(unnamed);

                    accumulated_dependencies.extend(resource_infos.iter().map(|res| {
                        CompiledResource {
                            path: res.compiled_path.clone(),
                            content_id: res.compiled_content_id.clone(),
                        }
                    }));
                    accumulated_dependencies.sort();
                    accumulated_dependencies.dedup();

                    assert_eq!(
                        compiled_resources
                            .iter()
                            .filter(|&info| resource_infos.iter().any(|a| a == info))
                            .count(),
                        0,
                        "duplicate compilation output detected"
                    );

                    compiled_resources.extend(resource_infos);
                    compiled_references.extend(resource_references);
                    compile_stats.extend(stats);

                    compiled.insert(node_index);
                }
                Err((node_index, e)) => {
                    error!("Compilation of '{:?}' failed {}", node_index, e);
                    compiled.insert(node_index);
                }
            };

            work = remaining;
        }

        Ok(CompileOutput {
            resources: compiled_resources,
            references: compiled_references,
            statistics: compile_stats,
        })
    }

    async fn link_work(
        &self,
        resource: &CompiledResourceInfo,
        references: &[CompiledResourceReference],
    ) -> Result<CompiledResource, Error> {
        info!("Linking {:?} ...", resource);
        let checksum = if let Some(checksum) = self
            .output_index
            .find_linked(
                resource.compiled_path.clone(),
                resource.context_hash,
                resource.source_hash,
            )
            .await?
        {
            checksum
        } else {
            //
            // for now, every derived resource gets an `assetfile` representation.
            //
            let asset_id = resource.compiled_path.resource_id();

            let resource_list = std::iter::once((asset_id, resource.compiled_content_id.clone()));
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

            let output = write_assetfile(
                resource_list,
                reference_list,
                &self.source_index.content_store,
            )
            .await?;

            let checksum = {
                async_span_scope!("content_store");
                self.source_index
                    .content_store
                    .write_resource_from_bytes(&output)
                    .await?
            };
            self.output_index
                .insert_linked(
                    resource.compiled_path.clone(),
                    resource.context_hash,
                    resource.source_hash,
                    checksum.clone(),
                )
                .await?;
            checksum
        };

        let asset_file = CompiledResource {
            path: resource.compiled_path.clone(),
            content_id: checksum,
        };
        Ok(asset_file)
    }

    /// Create asset files in runtime format containing compiled resources that
    /// include reference (load-time dependency) information
    /// based on provided compilation information.
    /// Currently each resource is linked into a separate *asset file*.
    async fn link(
        &mut self,
        resources: &[CompiledResourceInfo],
        references: &[CompiledResourceReference],
    ) -> Result<Vec<CompiledResource>, Error> {
        let timer = std::time::Instant::now();

        #[allow(clippy::type_complexity)]
        let work: Vec<
            Pin<Box<dyn Future<Output = Result<CompiledResource, Error>> + Send>>,
        > = resources
            .iter()
            .map(|resource| {
                async {
                    let link_timer = std::time::Instant::now();

                    let asset_file = self.link_work(resource, references).await?;
                    info!(
                        "Linked {} into: {} in {:?}",
                        resource.compiled_path,
                        asset_file.content_id,
                        link_timer.elapsed()
                    );
                    Ok(asset_file)
                }
                .boxed()
            })
            .collect::<Vec<_>>();

        let resource_files = try_join_all(work).await?;

        info!("Linking ended in {:?}.", timer.elapsed());

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

    /// Returns data content provider
    pub fn get_provider(&self) -> &Provider {
        &self.data_content_provider
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
