//! Data compiler interface.
//!
//! Data compiler is a binary that takes as input a
//! [`lgn_data_runtime::Resource`] and Resources it depends on and produces
//! one or more [`lgn_data_runtime::Resource`]s that are stored in a
//! [`ContentStore`]. As a results it creates new or updates existing
//! [`Manifest`] file containing metadata about the derived resources.
//!
//! [`compiler_api`] allows to structure *data compiler* in a specific way.
//!
//! # Data Compiler's main()
//!
//! A data compiler binary must use a [`compiler_main`] function provided by
//! this module. The signature requires data compiler to provide a static
//! [`CompilerDescriptor`] structure defining the properties of the data
//! compiler.
//!
//! Below you can see a minimum code required to compile a data compiler:
//!
//! ```no_run
//! # use lgn_data_compiler::{Locale, Platform, Target};
//! # use lgn_data_compiler::compiler_api::{Compiler, CompilerHash, CompilationEnv, DATA_BUILD_VERSION, compiler_main, CompilerContext, CompilerDescriptor, CompilationOutput, CompilerError};
//! # use lgn_data_runtime::{AssetRegistryOptions, ResourceType, ResourcePathId, Transform};
//! # use std::path::Path;
//! # use async_trait::async_trait;
//! # const INPUT_TYPE: ResourceType = ResourceType::new(b"src");
//! # const OUTPUT_TYPE: ResourceType = ResourceType::new(b"dst");
//! static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
//!    name: env!("CARGO_CRATE_NAME"),
//!    build_version: DATA_BUILD_VERSION,
//!    code_version: "",
//!    data_version: "",
//!    transform: &Transform::new(INPUT_TYPE, OUTPUT_TYPE),
//!    compiler_creator: || Box::new(TestCompiler {}),
//! };
//!
//! struct TestCompiler();
//!
//! #[async_trait]
//! impl Compiler for TestCompiler {
//! async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
//!    todo!()
//! }
//!
//! async fn hash(
//!    &self,
//!    code: &'static str,
//!    data: &'static str,
//!    env: &CompilationEnv,
//! ) -> CompilerHash {
//!    todo!()
//! }
//!
//! async fn compile(&self, context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
//!    todo!()
//! }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!    std::process::exit(match compiler_main(&std::env::args(), &COMPILER_INFO).await {
//!        Ok(_) => 0,
//!        Err(_) => 1,
//!    });
//! }
//! ```
//!
//! [`lgn_data_build`]: ../../lgn_data_build/index.html
//! [`compiler_api`]: ../compiler_api/index.html
//! [`ContentStore`]: ../content_store/index.html
//! [`Manifest`]: ../struct.Manifest.html

// This disables the lint crate-wide as a workaround to allow the doc above.
#![allow(clippy::needless_doctest_main)]

use std::{
    convert::Infallible,
    env,
    ffi::OsString,
    io::{stdout, Write},
    path::StripPrefixError,
    str::FromStr,
    sync::Arc,
};

use async_trait::async_trait;
use clap::{Parser, Subcommand};
use lgn_content_store::{
    indexing::{ResourceWriter, SharedTreeIdentifier},
    Config, Provider,
};
use lgn_data_model::ReflectionError;
use lgn_data_runtime::{
    manifest::ManifestId, AssetRegistry, AssetRegistryError, AssetRegistryOptions, ResourcePathId,
    ResourceProcessorError, Transform,
};
use serde::{Deserialize, Serialize};

use crate::{
    compiler_cmd::{
        CompilerHashCmdOutput, CompilerInfoCmdOutput, COMMAND_ARG_DER_DEPS, COMMAND_ARG_LOCALE,
        COMMAND_ARG_OFFLINE_MANIFEST_ID, COMMAND_ARG_PLATFORM, COMMAND_ARG_SRC_DEPS,
        COMMAND_ARG_TARGET, COMMAND_ARG_TRANSFORM, COMMAND_NAME_COMPILE,
        COMMAND_NAME_COMPILER_HASH, COMMAND_NAME_INFO,
    },
    compiler_node::{CompilerNode, CompilerRegistry, CompilerRegistryOptions},
    CompiledResource, CompiledResources, Locale, Platform, Target,
};

/// Current version of data pipeline.
///
/// > **NOTE**: This does not follow *Semantic Versioning* rules.
///
/// Changing `DATA_BUILD_VERSION` will:
///
/// * Invalidate all `data compilers`.
/// * Invalidate all `build index` files.
pub const DATA_BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");

/// *Data Compiler's* output.
///
/// Includes data which allows to load and validate
/// [`lgn_data_runtime::Resource`]s stored in [`ContentStore`]. As well as
/// references between resources that define load-time dependencies.
///
/// [`ContentStore`]: ../content_store/index.html
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompilationOutput {
    /// List of compiled resource's metadata.
    pub compiled_resources: Vec<CompiledResource>,
    /// List of references between compiled resources.
    pub resource_references: Vec<(ResourcePathId, ResourcePathId)>,
}

impl CompilationOutput {
    /// Create the command from a .json string.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CompilerError> {
        serde_json::from_slice::<Self>(bytes).map_err(CompilerError::SerdeJson)
    }

    /// Serialize the command into a .json string.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

/// The compilation environment - the context in which compilation runs.
pub struct CompilationEnv {
    /// Output build target type.
    pub target: Target,
    /// Output platform.
    pub platform: Platform,
    /// Output language/region.
    pub locale: Locale,
}

/// Compiler information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompilerInfo {
    /// Data build version of data compiler.
    pub build_version: String,
    /// Code version of data compiler.
    pub code_version: String,
    /// Resource and Asset data version.
    pub data_version: String,
    /// Transformation supported by data compiler.
    pub transform: Transform,
}

/// Context of the current compilation process.
pub struct CompilerContext<'a> {
    /// Compilation input - direct dependency of target.
    pub source: ResourcePathId,
    /// The desired compilation output (without the 'name' part).
    pub target_unnamed: ResourcePathId,
    /// Compilation dependency list.
    pub dependencies: &'a [ResourcePathId],
    /// Pre-configures asset registry builder.
    registry: Arc<AssetRegistry>,
    /// Compilation environment.
    pub env: &'a CompilationEnv,
    /// Content-addressable storage of compilation output.
    pub provider: &'a Provider,
}

impl CompilerContext<'_> {
    /// Returns asset registry responsible for loading resources.
    pub fn registry(&self) -> Arc<AssetRegistry> {
        self.registry.clone()
    }

    /// Stores `compiled_content` in the content store.
    ///
    /// Returned [`CompiledResource`] contains details about stored content.
    pub async fn store(
        &mut self,
        compiled_content: &[u8],
        path: ResourcePathId,
    ) -> Result<CompiledResource, CompilerError> {
        let content_id = self
            .provider
            .write_resource_from_bytes(compiled_content)
            .await?;
        Ok(CompiledResource { path, content_id })
    }

    /// Execute a workload on a separate thread yielding current future until the work completes.
    pub async fn execute_workload<F, R>(f: F) -> Result<R, CompilerError>
    where
        F: FnOnce() -> Result<R, CompilerError> + Send + 'static,
        R: Send + 'static,
    {
        tokio::task::spawn_blocking(f)
            .await
            .map_err(|_e| CompilerError::ExecuteWorkload)?
    }
}

/// Identifies a group of outputs generated by the data compiler.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
pub struct CompilerHash(pub u64);

/// The actual compiler API that should be implemented by users.
#[async_trait]
pub trait Compiler {
    /// Compiler initialization function.
    async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions;

    /// Function returning a `CompilerHash` for a given context.
    async fn hash(
        &self,
        code: &'static str,
        data: &'static str,
        env: &CompilationEnv,
    ) -> CompilerHash;

    /// Data compilation function.
    async fn compile(
        &self,
        context: CompilerContext<'_>,
    ) -> Result<CompilationOutput, CompilerError>;
}

/// Defines data compiler properties.
pub struct CompilerDescriptor {
    /// Compiler name
    pub name: &'static str,
    /// Data build version of data compiler.
    pub build_version: &'static str,
    /// Version of compiler's code.
    pub code_version: &'static str,
    /// Version of resource data formats.
    pub data_version: &'static str,
    /// Compiler supported resource transformation `f(.0)->.1`.
    pub transform: &'static Transform,
    /// Compiler factory.
    pub compiler_creator: fn() -> Box<dyn Compiler + Send + Sync>,
}

/// Compiler error.
#[derive(thiserror::Error, Debug)]
pub enum CompilerError {
    /// Cannot write to stdout.
    #[error("IOError")]
    StdoutError(#[from] std::io::Error),

    /// Invalid command line arguments.
    #[error("Invalid string while parsing")]
    Parse,
    /// Invalid command line arguments.
    #[error("Invalid Arguments")]
    InvalidArgs,
    /// Invalid resource id.
    #[error("Invalid Resource Id")]
    InvalidResourceId,
    /// Compiler not found for a given transform.
    #[error("Compiler transform '{0}' not found")]
    CompilerNotFound(Transform),
    /// Invalid input/output resource type pair.
    #[error("Invalid Transform")]
    InvalidTransform,
    /// Invalid ResourcePathId provided as input.
    #[error("Invalid ResourcePathId {0}")]
    InvalidResource(ResourcePathId),
    /// Unknown platform.
    #[error("Invalid Platform")]
    InvalidPlatform,
    /// Unknown target.
    #[error("Invalid Target")]
    InvalidTarget,
    /// Unknown command.
    #[error("Unknown Command")]
    UnknownCommand,

    /// Asset read/write failure.
    #[error("Asset Store Error")]
    AssetStoreError,
    /// IO failure.

    #[error("Serialization error with serde_json: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// AssetRegistry fallthrough
    #[error(transparent)]
    Reflection(#[from] ReflectionError),

    /// AssetRegistry fallthrough
    #[error(transparent)]
    ResourceProcessor(#[from] ResourceProcessorError),

    /// AssetRegistry fallthrough
    #[error(transparent)]
    AssetRegistry(#[from] AssetRegistryError),

    /// Infallible
    #[error(transparent)]
    Unreachable(#[from] Infallible),

    /// Strip
    #[error(transparent)]
    Strip(#[from] StripPrefixError),

    /// Compiler-specific compilation error.
    #[error("{0}")]
    CompilationError(String),

    /// Data executor error.
    #[error("{0}")]
    RemoteExecution(String),

    /// lgn-content-store errors.
    #[error(transparent)]
    CASError(#[from] lgn_content_store::Error),

    /// lgn-content-store indexing errors.
    #[error(transparent)]
    CASIndexingError(#[from] lgn_content_store::indexing::Error),

    /// Execute workload error.
    #[error("Execute workload failed")]
    ExecuteWorkload,
}

impl CompilerDescriptor {
    pub(crate) fn instantiate_compiler(&self) -> Box<dyn Compiler + Send + Sync> {
        (self.compiler_creator)()
    }

    pub(crate) async fn compiler_hash(
        &self,
        compiler: &(dyn Compiler + Send + Sync),
        env: &CompilationEnv,
    ) -> CompilerHash {
        compiler
            .hash(self.code_version, self.data_version, env)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn compile(
        &self,
        compiler: &(dyn Compiler + Send + Sync),
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        _derived_deps: &[CompiledResource],
        registry: Arc<AssetRegistry>,
        provider: &Provider,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        let transform = compile_path
            .last_transform()
            .ok_or(CompilerError::InvalidTransform)?;
        if self.transform != &transform {
            return Err(CompilerError::InvalidTransform);
        }

        assert!(!compile_path.is_named());
        let context = CompilerContext {
            source: compile_path.direct_dependency().unwrap(),
            target_unnamed: compile_path,
            dependencies,
            registry,
            env,
            provider,
        };

        compiler.compile(context).await
    }
}

async fn get_transform_hash(
    compilers: &CompilerRegistry,
    env: &CompilationEnv,
    transform: Transform,
) -> Result<CompilerHash, CompilerError> {
    let (compiler, transform) = compilers
        .find_compiler(transform)
        .ok_or(CompilerError::CompilerNotFound(transform))?;

    let compiler_hash = compiler
        .compiler_hash(transform, env)
        .await
        .map_err(|_e| CompilerError::CompilerNotFound(transform))?;

    Ok(compiler_hash)
}

async fn run(command: Commands, compilers: CompilerRegistry) -> Result<(), CompilerError> {
    match command {
        Commands::Info => {
            serde_json::to_writer_pretty(
                stdout(),
                &CompilerInfoCmdOutput::from_registry(&compilers),
            )?;
            Ok(())
        }
        Commands::CompilerHash {
            target,
            platform,
            locale,
            transform,
        } => {
            let target = Target::from_str(&target).map_err(|_e| CompilerError::InvalidPlatform)?;
            let platform =
                Platform::from_str(&platform).map_err(|_e| CompilerError::InvalidPlatform)?;
            let locale = Locale::new(&locale);
            let transform = transform
                .map(|transform| {
                    Transform::from_str(&transform).map_err(|_e| CompilerError::InvalidTransform)
                })
                .transpose()?;

            let env = CompilationEnv {
                target,
                platform,
                locale,
            };

            let compiler_hash_list = if let Some(transform) = transform {
                let compiler_hash = get_transform_hash(&compilers, &env, transform).await?;
                vec![(transform, compiler_hash)]
            } else {
                let mut compiler_hashes = vec![];
                for info in compilers.infos() {
                    let transform = info.transform;
                    let compiler_hash = get_transform_hash(&compilers, &env, transform).await?;
                    compiler_hashes.push((transform, compiler_hash));
                }

                compiler_hashes
            };

            let output = CompilerHashCmdOutput { compiler_hash_list };
            serde_json::to_writer_pretty(stdout(), &output)?;
            Ok(())
        }
        Commands::Compile {
            resource: resource_path,
            src_deps,
            der_deps,
            offline_manifest_id,
            target,
            platform,
            locale,
        } => {
            let derived = ResourcePathId::from_str(&resource_path)
                .map_err(|_e| CompilerError::InvalidResourceId)?;
            let target = Target::from_str(&target).map_err(|_e| CompilerError::InvalidPlatform)?;
            let platform =
                Platform::from_str(&platform).map_err(|_e| CompilerError::InvalidPlatform)?;
            let locale = Locale::new(&locale);
            let dependencies: Vec<ResourcePathId> = src_deps
                .iter()
                .filter_map(|s| ResourcePathId::from_str(s).ok())
                .collect();
            let derived_deps: Vec<CompiledResource> = der_deps
                .iter()
                .filter_map(|s| CompiledResource::from_str(s).ok())
                .collect();

            let env = CompilationEnv {
                target,
                platform,
                locale,
            };

            let transform = derived
                .last_transform()
                .ok_or_else(|| CompilerError::InvalidResource(derived.clone()))?;

            let data_provider = Arc::new(Config::load_and_instantiate_volatile_provider().await?);

            let runtime_manifest_id = {
                let manifest = CompiledResources {
                    compiled_resources: derived_deps.clone(),
                };

                let manifest = manifest
                    .into_rt_manifest(&data_provider, |_rpid| true)
                    .await;

                SharedTreeIdentifier::new(manifest)
            };

            let registry = {
                let (compiler, _) = compilers
                    .find_compiler(transform)
                    .ok_or(CompilerError::CompilerNotFound(transform))?;

                let registry = AssetRegistryOptions::new()
                    .add_device_cas(Arc::clone(&data_provider), runtime_manifest_id.clone()); // todo: filter dependencies only

                compiler.init(registry).await.create().await
            };

            let shell = CompilerNode::new(compilers, registry);

            let (compiler, _) = shell
                .compilers()
                .find_compiler(transform)
                .ok_or(CompilerError::CompilerNotFound(transform))?;

            let compilation_output = compiler
                .compile(
                    derived,
                    &dependencies,
                    &derived_deps,
                    shell.registry(),
                    &data_provider,
                    &runtime_manifest_id,
                    &env,
                )
                .await?;

            let output = CompilationOutput {
                compiled_resources: compilation_output.compiled_resources,
                resource_references: compilation_output.resource_references,
            };
            stdout().write_all(output.to_string().as_bytes())?;
            Ok(())
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "TODO: compiler name")]
#[clap(about = "Data compiler", version, author)]
#[clap(arg_required_else_help(true))]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Information about the compiler.
    #[clap(name = COMMAND_NAME_INFO)]
    Info,
    /// Compiler Hash list based on provided build context information.
    #[clap(name = COMMAND_NAME_COMPILER_HASH)]
    CompilerHash {
        /// Build target (Game, Server, etc).
        #[clap(long)]
        target: String,
        /// Build platform (Windows, Unix, etc).
        #[clap(long)]
        platform: String,
        /// Build localization (en, fr, etc).
        #[clap(long)]
        locale: String,
        /// Optional transformation to return hash for. Otherwise returns hashes for all transformations.
        #[clap(long = COMMAND_ARG_TRANSFORM)]
        transform: Option<String>,
    },
    /// Compile given resource.
    #[clap(name = COMMAND_NAME_COMPILE)]
    Compile {
        /// Resource to compile.
        resource: String,
        /// Source dependencies.
        #[clap(long = COMMAND_ARG_SRC_DEPS, multiple_values=true)]
        src_deps: Vec<String>,
        /// Derived dependencies.
        #[clap(long = COMMAND_ARG_DER_DEPS, multiple_values=true)]
        der_deps: Vec<String>,
        /// Offline manifest id
        #[clap(long = COMMAND_ARG_OFFLINE_MANIFEST_ID)]
        offline_manifest_id: TreeIdentifier,
        /// Build target (Game, Server, etc).
        #[clap(long = COMMAND_ARG_TARGET)]
        target: String,
        /// Build platform (Windows, Unix, etc).
        #[clap(long = COMMAND_ARG_PLATFORM)]
        platform: String,
        /// Build localization (en, fr, etc).
        #[clap(long = COMMAND_ARG_LOCALE)]
        locale: String,
    },
}

/// The main function of every data compiler.
///
/// This must be called by the data compiler. It will parse and validate command
/// line arguments and invoke the appropriate function on the
/// `CompilerDescriptor` interface. The result will be written out to stdout.
///
/// > **NOTE**: Data compiler must not write to stdout because this could break
/// the specific output that is expected.
pub async fn compiler_main(
    _args: &env::Args,
    descriptor: &'static CompilerDescriptor,
) -> Result<(), CompilerError> {
    let compilers = CompilerRegistryOptions::default().add_compiler(descriptor);

    multi_compiler_main(compilers).await
}

/// Same as `compiler_main` but supports many compilers in a single binary.
pub async fn multi_compiler_main(compilers: CompilerRegistryOptions) -> Result<(), CompilerError> {
    let args: Vec<OsString> = env::args().map(OsString::from).collect();

    // Read the command line as a .json from stdin, if it isn't empty.
    /*if atty::is(atty::Stream::Stdin) {
        let mut input = String::new();
        if io::stdin().lock().read_to_string(&mut input)? > 0 {
            if let Ok(stdin_args) = CommandBuilder::from_bytes(input.as_bytes()) {
                args = stdin_args.to_os_args();
            }
        }
    }*/

    let args = Cli::try_parse_from(args).map_err(|err| {
        eprintln!("{}", err);
        CompilerError::InvalidArgs
    })?;

    let compilers = compilers.create().await;

    let result = run(args.command, compilers).await;
    if let Err(error) = &result {
        eprintln!("{}", error);
    }
    result
}
