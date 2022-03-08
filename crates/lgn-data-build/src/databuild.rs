use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use std::{env, io};

use lgn_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use lgn_data_compiler::compiler_api::{CompilationEnv, CompilationOutput, DATA_BUILD_VERSION};
use lgn_data_compiler::compiler_node::{CompilerNode, CompilerRegistry, CompilerStub};
use lgn_data_compiler::CompilerHash;
use lgn_data_compiler::{CompiledResource, CompiledResources};
use lgn_data_offline::Transform;
use lgn_data_offline::{resource::Project, ResourcePathId};
use lgn_data_runtime::manifest::Manifest;
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions, ResourceTypeAndId};
use lgn_tracing::{info, span_scope};
use lgn_utils::{DefaultHash, DefaultHasher};
use petgraph::{algo, Graph};

use crate::asset_file_writer::write_assetfile;
use crate::output_index::{
    AssetHash, CompiledResourceInfo, CompiledResourceReference, OutputIndex,
};
use crate::source_index::SourceIndex;
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
#[derive(Debug)]
pub struct DataBuild {
    source_index: SourceIndex,
    output_index: OutputIndex,
    resource_dir: PathBuf,
    content_store: HddContentStore,
    compilers: CompilerNode,
}

impl DataBuild {
    fn default_asset_registry(
        resource_dir: &Path,
        cas_addr: ContentStoreAddr,
        compilers: &CompilerRegistry,
        manifest: Option<Manifest>,
    ) -> Result<Arc<AssetRegistry>, Error> {
        let source_store = HddContentStore::open(cas_addr).ok_or(Error::InvalidContentStore)?;
        let manifest = manifest.unwrap_or_default();

        let mut options = AssetRegistryOptions::new()
            .add_device_cas(Box::new(source_store), manifest)
            .add_device_dir(resource_dir);

        options = compilers.init_all(options);

        Ok(options.create())
    }

    pub(crate) async fn new(config: DataBuildOptions, project: &Project) -> Result<Self, Error> {
        let content_store = HddContentStore::open(config.contentstore_path.clone())
            .ok_or(Error::InvalidContentStore)?;

        let source_index = SourceIndex::new(Box::new(content_store.clone()));

        let output_index = OutputIndex::create_new(OutputIndex::database_uri(
            &config.buildindex_dir,
            Self::version(),
        ))
        .await?;

        let compilers = config.compiler_options.create();
        let registry = config.registry.map_or_else(
            || {
                Self::default_asset_registry(
                    &project.resource_dir(),
                    config.contentstore_path.clone(),
                    &compilers,
                    config.manifest,
                )
            },
            Ok,
        )?;

        Ok(Self {
            source_index,
            output_index,
            resource_dir: project.resource_dir(),
            content_store,
            compilers: CompilerNode::new(compilers, registry),
        })
    }

    pub(crate) async fn open(config: DataBuildOptions, project: &Project) -> Result<Self, Error> {
        let content_store = HddContentStore::open(config.contentstore_path.clone())
            .ok_or(Error::InvalidContentStore)?;

        let source_index = SourceIndex::new(Box::new(content_store.clone()));
        let output_index = OutputIndex::open(OutputIndex::database_uri(
            &config.buildindex_dir,
            Self::version(),
        ))
        .await?;

        let compilers = config.compiler_options.create();
        let registry = config.registry.map_or_else(
            || {
                Self::default_asset_registry(
                    &project.resource_dir(),
                    config.contentstore_path.clone(),
                    &compilers,
                    config.manifest,
                )
            },
            Ok,
        )?;

        Ok(Self {
            source_index,
            output_index,
            resource_dir: project.resource_dir(),
            content_store,
            compilers: CompilerNode::new(compilers, registry),
        })
    }

    pub(crate) async fn open_or_create(
        config: DataBuildOptions,
        project: &Project,
    ) -> Result<Self, Error> {
        let content_store = HddContentStore::open(config.contentstore_path.clone())
            .ok_or(Error::InvalidContentStore)?;

        let source_index = SourceIndex::new(Box::new(content_store.clone()));

        let output_index = match OutputIndex::open(OutputIndex::database_uri(
            &config.buildindex_dir,
            Self::version(),
        ))
        .await
        {
            Ok(output_index) => Ok(output_index),
            Err(Error::NotFound) => {
                OutputIndex::create_new(OutputIndex::database_uri(
                    &config.buildindex_dir,
                    Self::version(),
                ))
                .await
            }
            Err(e) => Err(e),
        }?;

        let compilers = config.compiler_options.create();
        let registry = config.registry.map_or_else(
            || {
                Self::default_asset_registry(
                    &project.resource_dir(),
                    config.contentstore_path.clone(),
                    &compilers,
                    config.manifest,
                )
            },
            Ok,
        )?;

        Ok(Self {
            source_index,
            output_index,
            resource_dir: project.resource_dir(),
            content_store,
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
    /// Compilation results are stored in
    /// [`ContentStore`](`lgn_content_store::ContentStore`) specified in
    /// [`DataBuildOptions`] used to create this `DataBuild`.
    ///
    /// Provided `target`, `platform` and `locale` define the compilation
    /// context that can yield different compilation results.
    pub async fn compile(
        &mut self,
        compile_path: ResourcePathId,
        env: &CompilationEnv,
    ) -> Result<CompiledResources, Error> {
        self.compile_with_manifest(compile_path, env, None).await
    }

    /// Same as `compile` but it updates the `manifest` provided as an argument.
    pub async fn compile_with_manifest(
        &mut self,
        compile_path: ResourcePathId,
        env: &CompilationEnv,
        intermediate_output: Option<&Manifest>,
    ) -> Result<CompiledResources, Error> {
        self.output_index.record_pathid(&compile_path).await?;
        let mut result = CompiledResources::default();

        let start = std::time::Instant::now();
        info!("Compilation of {} Started", compile_path);

        let CompileOutput {
            resources,
            references,
            statistics: _stats,
        } = self
            .compile_path(compile_path, env, intermediate_output)
            .await?;

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

        info!("Compilation Ended ({}ms)", start.elapsed().as_millis());
        Ok(result)
    }

    /// Compile `compile_node` of the build graph and update *build index* one
    /// or more compilation results.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    async fn compile_node(
        output_index: &mut OutputIndex,
        cas_addr: ContentStoreAddr,
        project_dir: &Path,
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
                        cas_addr,
                        project_dir,
                        env,
                    )
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
                            compiled_checksum: resource.checksum,
                            compiled_size: resource.size,
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
    pub fn print_build_graph(
        &self,
        compile_path: ResourcePathId,
        name_parser: impl Fn(&ResourcePathId) -> String,
    ) -> String {
        if let Some(source_index) = self.source_index.current() {
            let build_graph = source_index.generate_build_graph(compile_path);
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
        } else {
            String::new()
        }
    }

    /// Compile a resource identified by [`ResourcePathId`] and all its
    /// dependencies and update the *build index* with compilation results.
    /// Returns a list of (id, checksum, size) of created resources and
    /// information about their dependencies. The returned results can be
    /// accessed by  [`lgn_content_store::ContentStore`] specified in
    /// [`DataBuildOptions`] used to create this `DataBuild`.
    // TODO: The list might contain many versions of the same [`ResourceId`] compiled for many
    // contexts (platform, target, locale, etc).
    async fn compile_path(
        &mut self,
        compile_path: ResourcePathId,
        env: &CompilationEnv,
        intermediate_output: Option<&Manifest>,
    ) -> Result<CompileOutput, Error> {
        if self.source_index.current().is_none() {
            return Err(Error::SourceIndex);
        }
        let source_index = self.source_index.current().unwrap();

        let build_graph = source_index.generate_build_graph(compile_path);

        let topological_order: Vec<_> = algo::toposort(&build_graph, None).map_err(|_e| {
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

            unique_transforms
                .into_iter()
                .map(|(transform, res_path_id)| {
                    let (compiler, transform) = self
                        .compilers
                        .compilers()
                        .find_compiler(transform)
                        .ok_or(Error::CompilerNotFound(transform, res_path_id))?;
                    let compiler_hash = compiler
                        .compiler_hash(transform, env)
                        .map_err(|e| Error::Io(e.into()))?;
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
        let mut node_hash = HashMap::<_, (AssetHash, AssetHash)>::new();

        let mut compiled_at_node = HashMap::<ResourcePathId, _>::new();

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

                // check if the unnamed ResourcePathId has been already compiled and early out.
                if let Some(node_index) = compiled_at_node.get(&compile_node) {
                    node_hash.insert(compile_node_index, *node_hash.get(node_index).unwrap());
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
                let context_hash = compute_context_hash(transform, compiler_hash, Self::version());

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
                        let source = compiled
                            .iter()
                            .find(|&compiled| compiled.compiled_path == direct_dependency)
                            .unwrap_or_else(|| {
                                panic!("compilation output of: {}", direct_dependency)
                            });

                        // this is how we truncate the 128 bit long checksum
                        // and convert it to a 64 bit source_hash.
                        AssetHash::from(source.compiled_checksum.default_hash())
                    }
                };

                node_hash.insert(compile_node_index, (context_hash, source_hash));

                info!("Compiling {} ({:?}) ...", compile_node, expected_name);
                let start = std::time::Instant::now();

                let (resource_infos, resource_references, stats) = Self::compile_node(
                    &mut self.output_index,
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
                )
                .await?;

                info!(
                    "Compiling {} Ended ({}ms)",
                    compile_node,
                    start.elapsed().as_millis()
                );

                // update the CAS manifest with new content in order to make new resources
                // visible to the next compilation node
                // NOTE: right now all the resources are visible to all compilation nodes.
                if let Some(manifest) = &intermediate_output {
                    for r in &resource_infos {
                        manifest.insert(
                            r.compiled_path.resource_id(),
                            r.compiled_checksum,
                            r.compiled_size,
                        );
                    }
                }

                // registry must be updated to release any resources that are no longer referenced.
                self.compilers.registry().update();

                // we check if the expected named output was produced.
                if let Some(expected_name) = expected_name {
                    if !resource_infos.iter().any(|info| {
                        info.compiled_path
                            .name()
                            .map_or(false, |name| name == expected_name)
                    }) {
                        return Err(Error::OutputNotPresent(
                            compile_node,
                            expected_name.to_string(),
                        ));
                    }
                }

                accumulated_dependencies.extend(resource_infos.iter().map(|res| {
                    CompiledResource {
                        path: res.compiled_path.clone(),
                        checksum: res.compiled_checksum,
                        size: res.compiled_size,
                    }
                }));

                assert_eq!(
                    compiled_resources
                        .iter()
                        .filter(|&info| resource_infos.iter().any(|a| a == info))
                        .count(),
                    0,
                    "duplicate compilation output detected"
                );

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
    async fn link(
        &mut self,
        resources: &[CompiledResourceInfo],
        references: &[CompiledResourceReference],
    ) -> Result<Vec<CompiledResource>, Error> {
        let mut resource_files = Vec::with_capacity(resources.len());
        for resource in resources {
            info!("Linking {:?} ...", resource);
            let (checksum, size) = if let Some((checksum, size)) = self
                .output_index
                .find_linked(
                    resource.compiled_path.clone(),
                    resource.context_hash,
                    resource.source_hash,
                )
                .await?
            {
                (checksum, size)
            } else {
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

                let checksum = {
                    span_scope!("content_store");
                    self.content_store
                        .store(&output)
                        .ok_or(Error::InvalidContentStore)?
                };
                self.output_index
                    .insert_linked(
                        resource.compiled_path.clone(),
                        resource.context_hash,
                        resource.source_hash,
                        checksum,
                        output.len(),
                    )
                    .await?;
                (checksum, output.len())
            };

            let asset_file = CompiledResource {
                path: resource.compiled_path.clone(),
                checksum,
                size,
            };

            info!("Linked {} into: {}", resource.compiled_path, checksum);
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

#[cfg(test)]
#[path = "test_general.rs"]
mod test_general;

#[cfg(test)]
#[path = "test_source_pull.rs"]
mod test_source_pull;

#[cfg(test)]
#[path = "test_compile.rs"]
mod test_compile;
