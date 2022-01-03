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
//! # use lgn_data_compiler::{CompilerHash, Locale, Platform, Target};
//! # use lgn_data_compiler::compiler_api::{CompilationEnv, DATA_BUILD_VERSION, compiler_main, CompilerContext, CompilerDescriptor, CompilationOutput, CompilerError};
//! # use lgn_data_offline::{ResourcePathId, Transform};
//! # use lgn_data_runtime::ResourceType;
//! # use lgn_content_store::ContentStoreAddr;
//! # use std::path::Path;
//! # const INPUT_TYPE: ResourceType = ResourceType::new(b"src");
//! # const OUTPUT_TYPE: ResourceType = ResourceType::new(b"dst");
//! static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
//!    name: env!("CARGO_CRATE_NAME"),
//!    build_version: DATA_BUILD_VERSION,
//!    code_version: "",
//!    data_version: "",
//!    transform: &Transform::new(INPUT_TYPE, OUTPUT_TYPE),
//!    compiler_hash_func: compiler_hash,
//!    compile_func: compile,
//! };
//!
//! fn compiler_hash(
//!    code: &'static str,
//!    data: &'static str,
//!    env: &CompilationEnv,
//! ) -> CompilerHash {
//!    todo!()
//! }
//!
//! fn compile(context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
//!    todo!()
//! }
//!
//! fn main() {
//!    std::process::exit(match compiler_main(std::env::args(), &COMPILER_INFO) {
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
    env,
    io::{self, stdout},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{AppSettings, Parser, Subcommand};
use lgn_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::AssetRegistryOptions;
use serde::{Deserialize, Serialize};

use crate::{
    compiler_cmd::{
        CompilerCompileCmdOutput, CompilerHashCmdOutput, CompilerInfoCmdOutput,
        COMMAND_ARG_COMPILED_ASSET_STORE, COMMAND_ARG_DER_DEPS, COMMAND_ARG_LOCALE,
        COMMAND_ARG_PLATFORM, COMMAND_ARG_RESOURCE_DIR, COMMAND_ARG_SRC_DEPS, COMMAND_ARG_TARGET,
        COMMAND_NAME_COMPILE, COMMAND_NAME_COMPILER_HASH, COMMAND_NAME_INFO,
    },
    CompiledResource, CompilerHash, Locale, Manifest, Platform, Target,
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
#[derive(Debug)]
pub struct CompilationOutput {
    /// List of compiled resource's metadata.
    pub compiled_resources: Vec<CompiledResource>,
    /// List of references between compiled resources.
    pub resource_references: Vec<(ResourcePathId, ResourcePathId)>,
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
#[derive(Serialize, Deserialize, Debug)]
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
    resources: Option<AssetRegistryOptions>,
    /// Compilation environment.
    pub env: &'a CompilationEnv,
    /// Content-addressable storage of compilation output.
    output_store: &'a mut dyn ContentStore,
}

impl CompilerContext<'_> {
    /// Returns options that can be used to create asset registry.
    pub fn take_registry(&mut self) -> AssetRegistryOptions {
        self.resources.take().unwrap()
    }

    /// Stores `compiled_content` in the content store.
    ///
    /// Returned [`CompiledResource`] contains details about stored content.
    pub fn store(
        &mut self,
        compiled_content: &[u8],
        path: ResourcePathId,
    ) -> Result<CompiledResource, CompilerError> {
        let checksum = self
            .output_store
            .store(compiled_content)
            .ok_or(CompilerError::AssetStoreError)?;
        Ok(CompiledResource {
            path,
            checksum,
            size: compiled_content.len(),
        })
    }
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
    /// Function returning a list of `CompilerHash` for a given context.
    pub compiler_hash_func:
        fn(code: &'static str, data: &'static str, env: &CompilationEnv) -> CompilerHash,
    /// Data compilation function.
    #[allow(clippy::type_complexity)]
    pub compile_func: fn(context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError>,
}

/// Compiler error.
#[derive(Debug)]
pub enum CompilerError {
    /// Cannot write to stdout.
    StdoutError,
    /// Invalid command line arguments.
    InvalidArgs,
    /// Invalid resource id.
    InvalidResourceId,
    /// Resource not found.
    ResourceNotFound,
    /// Invalid input/output resource type pair.
    InvalidTransform,
    /// Unknown platform.
    InvalidPlatform,
    /// Unknown target.
    InvalidTarget,
    /// Unknown command.
    UnknownCommand,
    /// Asset read/write failure.
    AssetStoreError,
    /// IO failure.
    ResourceReadFailed(io::Error),
    /// IO failure.
    ResourceWriteFailed(io::Error),
    /// Compiler-specific compilation error.
    CompilationError(&'static str),
}

impl std::error::Error for CompilerError {}
impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self {
            CompilerError::StdoutError => write!(f, "IOError"),
            CompilerError::InvalidArgs => write!(f, "InvalidArgs"),
            CompilerError::InvalidResourceId => write!(f, "InvalidResourceId"),
            CompilerError::ResourceNotFound => write!(f, "ResourceNotFound"),
            CompilerError::InvalidTransform => write!(f, "InvalidResourceType"),
            CompilerError::InvalidTarget => write!(f, "InvalidTarget"),
            CompilerError::InvalidPlatform => write!(f, "InvalidPlatform"),
            CompilerError::UnknownCommand => write!(f, "UnknownCommand"),
            CompilerError::AssetStoreError => write!(f, "AssetStoreError"),
            CompilerError::ResourceReadFailed(_) => write!(f, "ResourceReadFailed"),
            CompilerError::ResourceWriteFailed(_) => write!(f, "ResourceWriteFailed"),
            CompilerError::CompilationError(content) => {
                write!(f, "CompilationError: '{}'", content)
            }
        }
    }
}

impl CompilerDescriptor {
    pub(crate) fn compiler_hash(&self, env: &CompilationEnv) -> CompilerHash {
        (self.compiler_hash_func)(self.code_version, self.data_version, env)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        cas_addr: ContentStoreAddr,
        resource_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        let transform = compile_path
            .last_transform()
            .ok_or(CompilerError::InvalidTransform)?;
        if self.transform != &transform {
            return Err(CompilerError::InvalidTransform);
        }

        let source_store =
            HddContentStore::open(cas_addr.clone()).ok_or(CompilerError::AssetStoreError)?;
        let mut output_store =
            HddContentStore::open(cas_addr).ok_or(CompilerError::AssetStoreError)?;
        let manifest = Manifest {
            compiled_resources: derived_deps.to_owned(),
        };

        /*
        eprintln!("# Target: {}({})", derived, derived.resource_id());
        if let Some(source) = derived.direct_dependency() {
            eprintln!("# Source: {}({})", source, source.resource_id());
        }
        for derived_input in &manifest.compiled_resources {
            eprintln!(
                "# Derived Input: {}({}) chk: {} size: {}",
                derived_input.path,
                derived_input.path.resource_id(),
                derived_input.checksum,
                derived_input.size
            );
        }

        eprintln!("# Resource Dir: {:?}", &resource_dir);
        let paths = std::fs::read_dir(&resource_dir).unwrap();
        for path in paths {
            eprintln!("## File: {}", path.unwrap().path().display());
        }

        eprintln!("# CAS Dir: {:?}", &cas_dir);
        let paths = std::fs::read_dir(&cas_dir).unwrap();
        for path in paths {
            eprintln!("## File: {}", path.unwrap().path().display());
        }
        */

        let manifest = manifest.into_rt_manifest(|_rpid| true);

        let registry = AssetRegistryOptions::new()
            .add_device_cas(Box::new(source_store), manifest)
            .add_device_dir(resource_dir); // todo: filter dependencies only

        assert!(!compile_path.is_named());
        let context = CompilerContext {
            source: compile_path.direct_dependency().unwrap(),
            target_unnamed: compile_path,
            dependencies,
            resources: Some(registry),
            env,
            output_store: &mut output_store,
        };

        (self.compile_func)(context)
    }
}

fn run(command: Commands, descriptor: &CompilerDescriptor) -> Result<(), CompilerError> {
    match command {
        Commands::Info => {
            serde_json::to_writer_pretty(
                stdout(),
                &CompilerInfoCmdOutput::from_descriptor(descriptor),
            )
            .map_err(|_e| CompilerError::StdoutError)?;
            Ok(())
        }
        Commands::CompilerHash {
            target,
            platform,
            locale,
        } => {
            let target = Target::from_str(&target).map_err(|_e| CompilerError::InvalidPlatform)?;
            let platform =
                Platform::from_str(&platform).map_err(|_e| CompilerError::InvalidPlatform)?;
            let locale = Locale::new(&locale);

            let env = CompilationEnv {
                target,
                platform,
                locale,
            };
            let compiler_hash = descriptor.compiler_hash(&env);

            let output = CompilerHashCmdOutput { compiler_hash };
            serde_json::to_writer_pretty(stdout(), &output)
                .map_err(|_e| CompilerError::StdoutError)?;
            Ok(())
        }
        Commands::Compile {
            resource: resource_path,
            src_deps,
            der_deps,
            resource_dir,
            compiled_asset_store,
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
            let cas_addr = ContentStoreAddr::from(compiled_asset_store.as_str());

            let env = CompilationEnv {
                target,
                platform,
                locale,
            };

            let compilation_output = descriptor.compile(
                derived,
                &dependencies,
                &derived_deps,
                cas_addr,
                &resource_dir,
                &env,
            )?;

            let output = CompilerCompileCmdOutput {
                compiled_resources: compilation_output.compiled_resources,
                resource_references: compilation_output.resource_references,
            };
            serde_json::to_writer_pretty(stdout(), &output)
                .map_err(|_e| CompilerError::StdoutError)?;
            Ok(())
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "TODO: compiler name")]
#[clap(about = "CLI to query a local telemetry data lake", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
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
        /// Resource directory.
        #[clap(long = COMMAND_ARG_RESOURCE_DIR)]
        resource_dir: PathBuf,
        /// Compiled asset store.
        #[clap(long = COMMAND_ARG_COMPILED_ASSET_STORE)]
        compiled_asset_store: String,
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
// TODO: remove the limitation above.
pub fn compiler_main(
    args: env::Args,
    descriptor: &CompilerDescriptor,
) -> Result<(), CompilerError> {
    let args = Cli::try_parse_from(args).map_err(|err| {
        eprintln!("{}", err);
        CompilerError::InvalidArgs
    })?;
    let result = run(args.command, descriptor);
    if let Err(error) = &result {
        eprintln!("Compiler Failed With: '{:?}'", error);
    }
    result
}
