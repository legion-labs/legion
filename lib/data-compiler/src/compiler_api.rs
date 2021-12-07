//! Data compiler interface.
//!
//! Data compiler is a binary that takes as input a [`lgn_data_runtime::Resource`] and Resources it depends on and produces
//! one or more [`lgn_data_runtime::Resource`]s that are stored in a [`ContentStore`]. As a results it creates new
//! or updates existing [`Manifest`] file containing metadata about the derived resources.
//!
//! [`compiler_api`] allows to structure *data compiler* in a specific way.
//!
//! # Data Compiler's main()
//!
//! A data compiler binary must use a [`compiler_main`] function provided by this module.
//! The signature requires data compiler to provide a static [`CompilerDescriptor`] structure defining
//! the properties of the data compiler.
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
//!) -> CompilerHash {
//!    todo!()
//!}
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
    path::PathBuf,
    str::FromStr,
};

use clap::{AppSettings, Arg, ArgMatches, SubCommand};
use lgn_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::AssetRegistryOptions;

use crate::{
    compiler_cmd::{
        CompilerCompileCmdOutput, CompilerHashCmdOutput, CompilerInfoCmdOutput,
        COMMAND_ARG_COMPILED_ASSET_STORE, COMMAND_ARG_DER_DEPS, COMMAND_ARG_LOCALE,
        COMMAND_ARG_PLATFORM, COMMAND_ARG_RESOURCE_DIR, COMMAND_ARG_RESOURCE_PATH,
        COMMAND_ARG_SRC_DEPS, COMMAND_ARG_TARGET, COMMAND_NAME_COMPILE, COMMAND_NAME_COMPILER_HASH,
        COMMAND_NAME_INFO,
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
/// Includes data which allows to load and validate [`lgn_data_runtime::Resource`]s stored in [`ContentStore`].
/// As well as references between resources that define load-time dependencies.
///
/// [`ContentStore`]: ../content_store/index.html
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
    pub env: CompilationEnv,
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

fn run(matches: &ArgMatches<'_>, descriptor: &CompilerDescriptor) -> Result<(), CompilerError> {
    match matches.subcommand() {
        (COMMAND_NAME_INFO, _) => {
            serde_json::to_writer_pretty(
                stdout(),
                &CompilerInfoCmdOutput::from_descriptor(descriptor),
            )
            .map_err(|_e| CompilerError::StdoutError)?;
            Ok(())
        }
        (COMMAND_NAME_COMPILER_HASH, Some(cmd_args)) => {
            let target = cmd_args.value_of(COMMAND_ARG_TARGET).unwrap();
            let platform = cmd_args.value_of(COMMAND_ARG_PLATFORM).unwrap();
            let locale = cmd_args.value_of(COMMAND_ARG_LOCALE).unwrap();

            let target = Target::from_str(target).map_err(|_e| CompilerError::InvalidPlatform)?;
            let platform =
                Platform::from_str(platform).map_err(|_e| CompilerError::InvalidPlatform)?;
            let locale = Locale::new(locale);

            let env = CompilationEnv {
                target,
                platform,
                locale,
            };

            let compiler_hash = (descriptor.compiler_hash_func)(
                descriptor.code_version,
                descriptor.data_version,
                &env,
            );
            let output = CompilerHashCmdOutput { compiler_hash };
            serde_json::to_writer_pretty(stdout(), &output)
                .map_err(|_e| CompilerError::StdoutError)?;
            Ok(())
        }
        (COMMAND_NAME_COMPILE, Some(cmd_args)) => {
            let derived = cmd_args.value_of(COMMAND_ARG_RESOURCE_PATH).unwrap();
            let target = cmd_args.value_of(COMMAND_ARG_TARGET).unwrap();
            let platform = cmd_args.value_of(COMMAND_ARG_PLATFORM).unwrap();
            let locale = cmd_args.value_of(COMMAND_ARG_LOCALE).unwrap();

            let derived =
                ResourcePathId::from_str(derived).map_err(|_e| CompilerError::InvalidResourceId)?;
            let target = Target::from_str(target).map_err(|_e| CompilerError::InvalidPlatform)?;
            let platform =
                Platform::from_str(platform).map_err(|_e| CompilerError::InvalidPlatform)?;
            let locale = Locale::new(locale);
            let dependencies: Vec<ResourcePathId> = cmd_args
                .values_of(COMMAND_ARG_SRC_DEPS)
                .unwrap_or_default()
                .filter_map(|s| ResourcePathId::from_str(s).ok())
                .collect();
            let derived_deps: Vec<CompiledResource> = cmd_args
                .values_of(COMMAND_ARG_DER_DEPS)
                .unwrap_or_default()
                .filter_map(|s| CompiledResource::from_str(s).ok())
                .collect();
            let cas_dir = cmd_args.value_of(COMMAND_ARG_COMPILED_ASSET_STORE).unwrap();
            let asset_store_path = ContentStoreAddr::from(cas_dir);
            let resource_dir = PathBuf::from(cmd_args.value_of(COMMAND_ARG_RESOURCE_DIR).unwrap());

            let transform = derived
                .last_transform()
                .ok_or(CompilerError::InvalidTransform)?;
            if descriptor.transform != &transform {
                return Err(CompilerError::InvalidTransform);
            }

            let source_store = HddContentStore::open(asset_store_path.clone())
                .ok_or(CompilerError::AssetStoreError)?;
            let mut output_store =
                HddContentStore::open(asset_store_path).ok_or(CompilerError::AssetStoreError)?;
            let manifest = Manifest {
                compiled_resources: derived_deps,
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

            assert!(!derived.is_named());
            let context = CompilerContext {
                source: derived.direct_dependency().unwrap(),
                target_unnamed: derived,
                dependencies: &dependencies,
                resources: Some(registry),
                env: CompilationEnv {
                    target,
                    platform,
                    locale,
                },
                output_store: &mut output_store,
            };

            let compilation_output = (descriptor.compile_func)(context)?;

            let output = CompilerCompileCmdOutput {
                compiled_resources: compilation_output.compiled_resources,
                resource_references: compilation_output.resource_references,
            };
            serde_json::to_writer_pretty(stdout(), &output)
                .map_err(|_e| CompilerError::StdoutError)?;
            Ok(())
        }
        _ => Err(CompilerError::UnknownCommand),
    }
}

/// The main function of every data compiler.
///
/// This must be called by the data compiler. It will parse and validate command line arguments
/// and invoke the appropriate function on the `CompilerDescriptor` interface.
/// The result will be written out to stdout.
///
/// > **NOTE**: Data compiler must not write to stdout because this could break the specific output that is expected.
// TODO: remove the limitation above.
pub fn compiler_main(
    args: env::Args,
    descriptor: &CompilerDescriptor,
) -> Result<(), CompilerError> {
    let matches = clap::App::new("todo: compiler name")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(descriptor.code_version)
        .about("todo: about")
        .subcommand(
            SubCommand::with_name(COMMAND_NAME_INFO).about("Information about the compiler."),
        )
        .subcommand(
            SubCommand::with_name(COMMAND_NAME_COMPILER_HASH)
                .about("Compiler Hash list based on provided build context information.")
                .arg(
                    Arg::with_name(COMMAND_ARG_TARGET)
                        .required(true)
                        .takes_value(true)
                        .long(COMMAND_ARG_TARGET)
                        .help("Build target (Game, Server, etc)."),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_PLATFORM)
                        .required(true)
                        .takes_value(true)
                        .long(COMMAND_ARG_PLATFORM)
                        .help("Build platform (Windows, Unix, etc)"),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_LOCALE)
                        .required(true)
                        .takes_value(true)
                        .long(COMMAND_ARG_LOCALE)
                        .help("Build localization (en, fr, etc)"),
                ),
        )
        .subcommand(
            SubCommand::with_name(COMMAND_NAME_COMPILE)
                .about("Compile given resource.")
                .arg(
                    Arg::with_name(COMMAND_ARG_RESOURCE_PATH)
                        .required(true)
                        .help("Path in build graph to compile."),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_SRC_DEPS)
                        .takes_value(true)
                        .long(COMMAND_ARG_SRC_DEPS)
                        .multiple(true)
                        .help("Source dependencies to include."),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_DER_DEPS)
                        .takes_value(true)
                        .long(COMMAND_ARG_DER_DEPS)
                        .multiple(true)
                        .help("List of derived dependencies (id, hash, size)."),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_RESOURCE_DIR)
                        .takes_value(true)
                        .required(true)
                        .long(COMMAND_ARG_RESOURCE_DIR)
                        .help("Root resource directory."),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_COMPILED_ASSET_STORE)
                        .takes_value(true)
                        .long(COMMAND_ARG_COMPILED_ASSET_STORE)
                        .required(true)
                        .multiple(true)
                        .help("Content Store addresses where resources will be output."),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_TARGET)
                        .required(true)
                        .takes_value(true)
                        .long(COMMAND_ARG_TARGET)
                        .help("Build target (Game, Server, etc)."),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_PLATFORM)
                        .required(true)
                        .takes_value(true)
                        .long(COMMAND_ARG_PLATFORM)
                        .help("Build platform (Windows, Unix, etc)"),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_LOCALE)
                        .required(true)
                        .takes_value(true)
                        .long(COMMAND_ARG_LOCALE)
                        .help("Build localization (en, fr, etc)"),
                ),
        )
        .get_matches_from(args);

    let result = run(&matches, descriptor);
    if let Err(error) = &result {
        eprintln!("Compiler Failed With: '{:?}'", error);
    }
    result
}
