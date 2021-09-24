//! Data compiler interface.
//!
//! Data compiler is a binary that takes as input a [`legion_data_offline::resource::Resource`] and Resources it depends on and produces
//! one or more [`legion_data_runtime::Asset`]s that are stored in a [`ContentStore`]. As a results it creates new
//! or updates existing [`Manifest`] file containing metadata about the compiled Assets.
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
//! # use legion_data_compiler::{CompilerHash, Locale, Platform, Target};
//! # use legion_data_compiler::compiler_api::{DATA_BUILD_VERSION, compiler_main, CompilerContext, CompilerDescriptor, CompilationOutput, CompilerError};
//! # use legion_data_offline::{asset::AssetPathId, resource::ResourceType};
//! # use legion_data_runtime::AssetType;
//! # use legion_content_store::ContentStoreAddr;
//! # use std::path::Path;
//! # const INPUT_TYPE: ResourceType = ResourceType::new(b"src");
//! # const OUTPUT_TYPE: AssetType = AssetType::new(b"dst");
//! static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
//!    name: env!("CARGO_CRATE_NAME"),
//!    build_version: DATA_BUILD_VERSION,
//!    code_version: "",
//!    data_version: "",
//!    transform: &(INPUT_TYPE.content(), OUTPUT_TYPE.content()),
//!    compiler_hash_func: compiler_hash,
//!    compile_func: compile,
//! };
//!
//! fn compiler_hash(
//!    code: &'static str,
//!    data: &'static str,
//!    target: Target,
//!    platform: Platform,
//!    locale: &Locale,
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
//! # Reading Resources
//!
//! A **data compiler** is able to load and read certain *resources* that are available through [`CompilerContext`].
//!
//! The main *resource* is the leaf of the **build graph** being currently compiled. It is accessible by
//! calling [`AssetPathId::source_resource_path`] on [`CompilerContext::compile_path`].
//!
//! It can also access any *intermediate resource* that is part of the [`CompilerContext::compile_path`]'s [`AssetPathId`] provided that
//! it was whitelisted as a *derived dependency* during the compilation process.
//!
//! Reading source `Resources` is done with [`CompilerContext::load_resource`] function:
//!
//! ```no_run
//! # use legion_data_offline::{resource::{ResourceId, ResourceType, Resource, ResourceRegistryOptions, ResourceProcessor}, asset::AssetPathId};
//! # use legion_data_compiler::compiler_api::{CompilerContext, CompilationOutput, CompilerError};
//! # pub const SOURCE_GEOMETRY: ResourceType = ResourceType::new(b"src_geom");
//! # pub struct SourceGeomProc {}
//! # impl ResourceProcessor for SourceGeomProc {
//! # fn new_resource(&mut self) -> Box<(dyn Resource + 'static)> { todo!() }
//! # fn extract_build_dependencies(&mut self, _: &(dyn Resource + 'static)) -> Vec<AssetPathId> { todo!() }
//! # fn write_resource(&mut self, _: &(dyn Resource + 'static), _: &mut dyn std::io::Write) -> Result<usize, std::io::Error> { todo!() }
//! # fn read_resource(&mut self, _: &mut dyn std::io::Read) -> Result<Box<(dyn Resource + 'static)>, std::io::Error> { todo!() }
//! # }
//! fn compile(context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
//!   let mut registry = ResourceRegistryOptions::new()
//!     .add_type(SOURCE_GEOMETRY, Box::new(SourceGeomProc {}))
//!     .create_registry();
//!
//!   let resource = context.load_resource(&context.compile_path.source_resource_path(), &mut registry).expect("loaded resource");
//!   let resource = resource.get::<refs_resource::TestResource>(&registry).unwrap();
//! # todo!();
//!   // ...
//! }
//! ```
//!
//! > For more about `Assets` and `Resources` see [`legion_data_offline`] and [`legion_data_runtime`] crates.
//!
//! [`legion_data_build`]: ../../legion_data_build/index.html
//! [`compiler_api`]: ../compiler_api/index.html
//! [`ContentStore`]: ../content_store/index.html
//! [`Manifest`]: ../struct.Manifest.html

// This disables the lint crate-wide as a workaround to allow the doc above.
#![allow(clippy::needless_doctest_main)]

use crate::{
    compiler_cmd::{
        CompilerCompileCmdOutput, CompilerHashCmdOutput, CompilerInfoCmdOutput,
        COMMAND_ARG_COMPILED_ASSET_STORE, COMMAND_ARG_DER_DEPS, COMMAND_ARG_LOCALE,
        COMMAND_ARG_PLATFORM, COMMAND_ARG_RESOURCE_DIR, COMMAND_ARG_RESOURCE_PATH,
        COMMAND_ARG_SRC_DEPS, COMMAND_ARG_TARGET, COMMAND_NAME_COMPILE, COMMAND_NAME_COMPILER_HASH,
        COMMAND_NAME_INFO,
    },
    CompiledResource, CompilerHash, Locale, Platform, Target,
};
use clap::{AppSettings, Arg, ArgMatches, SubCommand};
use legion_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use legion_data_offline::{
    asset::AssetPathId,
    resource::{ResourceHandleUntyped, ResourceId, ResourceRegistry, ResourceType},
};
use legion_data_runtime::ContentType;
use std::{
    convert::TryFrom,
    env,
    fs::File,
    io::{self, stdout},
    path::{Path, PathBuf},
    str::FromStr,
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
/// Includes data which allows to load and validate [`legion_data_runtime::Asset`]s stored in [`ContentStore`].
/// As well as references between resources that define load-time dependencies.
///
/// [`ContentStore`]: ../content_store/index.html
pub struct CompilationOutput {
    /// List of compiled resource's metadata.
    pub compiled_resources: Vec<CompiledResource>,
    /// List of references between compiled resources.
    pub resource_references: Vec<(AssetPathId, AssetPathId)>,
}

/// Context of the current compilation process.
pub struct CompilerContext<'a> {
    /// The desired compilation output.
    pub compile_path: AssetPathId,
    /// Compilation dependency list.
    pub dependencies: &'a [AssetPathId],
    /// List of derived dependencies accumulated in this compilation pass.
    derived_deps: &'a [CompiledResource],
    /// Compilation target.
    pub target: Target,
    /// Compilation platform.
    pub platform: Platform,
    /// Compilation locale.
    pub locale: &'a Locale,
    /// Content-addressable storage of compilation output.
    pub content_store: &'a mut dyn ContentStore,
    /// Directory where `derived` and `dependencies` are assumed to be stored.
    // todo: this should be changed to ContentStore with filtering.
    pub resource_dir: &'a Path,
}

impl CompilerContext<'_> {
    /// Synchronously loads a resource.
    ///
    /// Resource `id` can be loaded in two different ways:
    /// * from [`Self::resource_dir`] in case of a *source resource*.
    /// * from [`Self::content_store`] in case of a *derived resource*
    ///
    /// Derived resource must appear in `derived_deps` list of derived dependencies to be successfully loaded.
    pub fn load_resource(
        &self,
        id: &AssetPathId,
        resources: &mut ResourceRegistry,
    ) -> Result<ResourceHandleUntyped, CompilerError> {
        if id.is_source() {
            let kind = ResourceType::try_from(id.content_type()).unwrap();
            //
            // for now, we only allow to load the `derived` resource's source.
            //
            // in the future we might want to load different leaves of this build path.
            // in this case we would need to additionally check `self.dependencies`.
            //
            if self.compile_path.source_resource() != id.source_resource() {
                return Err(CompilerError::ResourceNotFound);
            }
            let resource_path = resource_path(self.resource_dir, id.source_resource());
            let mut resource_file =
                File::open(resource_path).map_err(CompilerError::ResourceReadFailed)?;
            let handle = resources
                .deserialize_resource(kind, &mut resource_file)
                .map_err(CompilerError::ResourceReadFailed)?;
            Ok(handle)
        } else if let Some(derived) = self.derived_deps.iter().find(|&dep| &dep.path == id) {
            if let Some(content) = self.content_store.read(derived.checksum) {
                //
                // for now, only derived Resources can be loaded.
                // this should be extended to Assets but would require
                // a change in this fn's signature
                //
                let kind = ResourceType::try_from(id.content_type()).unwrap();
                Ok(resources
                    .deserialize_resource(kind, &mut &content[..])
                    .map_err(CompilerError::ResourceReadFailed)?)
            } else {
                Err(CompilerError::AssetStoreError)
            }
        } else {
            Err(CompilerError::ResourceNotFound)
        }
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
    pub transform: &'static (ContentType, ContentType),
    /// Function returning a list of `CompilerHash` for a given context.
    pub compiler_hash_func: fn(
        code: &'static str,
        data: &'static str,
        target: Target,
        platform: Platform,
        locale: &Locale,
    ) -> CompilerHash,
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

            let compiler_hash = (descriptor.compiler_hash_func)(
                descriptor.code_version,
                descriptor.data_version,
                target,
                platform,
                &locale,
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
                AssetPathId::from_str(derived).map_err(|_e| CompilerError::InvalidResourceId)?;
            let target = Target::from_str(target).map_err(|_e| CompilerError::InvalidPlatform)?;
            let platform =
                Platform::from_str(platform).map_err(|_e| CompilerError::InvalidPlatform)?;
            let locale = Locale::new(locale);
            let dependencies: Vec<AssetPathId> = cmd_args
                .values_of(COMMAND_ARG_SRC_DEPS)
                .unwrap_or_default()
                .filter_map(|s| AssetPathId::from_str(s).ok())
                .collect();
            let derived_deps: Vec<CompiledResource> = cmd_args
                .values_of(COMMAND_ARG_DER_DEPS)
                .unwrap_or_default()
                .filter_map(|s| CompiledResource::from_str(s).ok())
                .collect();
            let asset_store_path = ContentStoreAddr::from(
                cmd_args.value_of(COMMAND_ARG_COMPILED_ASSET_STORE).unwrap(),
            );
            let resource_dir = PathBuf::from(cmd_args.value_of(COMMAND_ARG_RESOURCE_DIR).unwrap());

            let transform = derived
                .last_transform()
                .ok_or(CompilerError::InvalidTransform)?;
            if descriptor.transform != &transform {
                return Err(CompilerError::InvalidTransform);
            }

            let mut content_store =
                HddContentStore::open(asset_store_path).ok_or(CompilerError::AssetStoreError)?;

            let context = CompilerContext {
                compile_path: derived,
                dependencies: &dependencies,
                derived_deps: &derived_deps,
                target,
                platform,
                locale: &locale,
                content_store: &mut content_store,
                resource_dir: &resource_dir,
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

fn resource_path(dir: &Path, id: ResourceId) -> PathBuf {
    dir.join(format!("{:x}", id))
}
