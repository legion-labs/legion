//! Data compiler interface.
//!
//! Data compiler is a binary that takes as input a [`legion_resources::Resource`] and Resources it depends on and produces
//! one or more [`legion_assets::Asset`]s that are stored in a [`CompiledAssetStore`]. As a results it creates new
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
//! # use legion_data_compiler::{CompiledAsset, CompilerHash, Locale, Platform, Target};
//! # use legion_data_compiler::compiler_api::{DATA_BUILD_VERSION, compiler_main, CompilerDescriptor, CompilerError};
//! # use legion_resources::ResourceId;
//! # use legion_data_compiler::compiled_asset_store::CompiledAssetStoreAddr;
//! # use std::path::Path;
//! static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
//!    build_version: DATA_BUILD_VERSION,
//!    code_version: "",
//!    data_version: "",
//!    resource_types: &[],
//!    compiler_hash_func: compiler_hash,
//!    compile_func: compile,
//! };
//!
//! fn compiler_hash(
//!    code: &'static str,
//!    data: &'static str,
//!    target: Target,
//!    platform: Platform,
//!    locale: Locale,
//!) -> Vec<CompilerHash> {
//!    todo!()
//!}
//!
//!fn compile(
//!    source: ResourceId,
//!    dependencies: &[ResourceId],
//!    target: Target,
//!    platform: Platform,
//!    locale: &Locale,
//!    compiled_asset_store_path: CompiledAssetStoreAddr,
//!    resource_dir: &Path,
//!) -> Result<Vec<CompiledAsset>, CompilerError> {
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
//! # Reading Resources and Writing Assets
//!
//! Compilation process transforms [`legion_resources::Resource`] into [`legion_assets::Asset`]s.
//!
//! Reading source `Resources` is done with [`compiler_load_resource`] function:
//!
//! ```no_run
//! # use legion_resources::{ResourceId, ResourceRegistry};
//! # use legion_resources::test_resource;
//! # use legion_data_compiler::compiler_api::compiler_load_resource;
//! # let source = ResourceId::from_raw(1).unwrap();
//! let mut registry = ResourceRegistry::default();
//! registry.register_type(test_resource::TYPE_ID, Box::new(test_resource::TestResourceProc {}));
//!
//! let resource = compiler_load_resource(source, "./resources/", &mut registry).expect("loaded resource");
//! let resource = resource.get::<test_resource::TestResource>(&registry).unwrap();
//! ```
//!
//! For more about `Assets` and `Resources` see [`legion_resources`] and [`legion_assets`] crates.
//!
//! [`legion_data_build`]: ../../legion_data_build/index.html
//! [`compiler_api`]: ../compiler_api/index.html
//! [`CompiledAssetStore`]: ../compiled_asset_store/index.html
//! [`Manifest`]: ../struct.Manifest.html

// This disables the lint crate-wide as a workaround to allow the doc above.
#![allow(clippy::needless_doctest_main)]

use crate::{
    compiled_asset_store::CompiledAssetStoreAddr,
    compiler_cmd::{
        CompilerCompileCmdOutput, CompilerHashCmdOutput, CompilerInfoCmdOutput,
        COMMAND_ARG_COMPILED_ASSET_STORE, COMMAND_ARG_DEPENDENCIES, COMMAND_ARG_LOCALE,
        COMMAND_ARG_PLATFORM, COMMAND_ARG_RESOURCE, COMMAND_ARG_RESOURCE_DIR, COMMAND_ARG_TARGET,
        COMMAND_NAME_COMPILE, COMMAND_NAME_COMPILER_HASH, COMMAND_NAME_INFO,
    },
    CompiledAsset, CompilerHash, Locale, Platform, Target,
};
use clap::{AppSettings, Arg, SubCommand};
use legion_resources::{ResourceHandle, ResourceId, ResourceRegistry, ResourceType, RESOURCE_EXT};
use std::{
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

/// Defines data compiler properties.
pub struct CompilerDescriptor {
    /// Data build version of data compiler.
    pub build_version: &'static str,
    /// Version of compiler's code.
    pub code_version: &'static str,
    /// Version of resource and asset data formats.
    pub data_version: &'static str,
    /// Compiler supported resource types.
    pub resource_types: &'static [ResourceType],
    /// Function returning a list of `CompilerHash` for a given context.
    pub compiler_hash_func: fn(
        code: &'static str,
        data: &'static str,
        target: Target,
        platform: Platform,
        locale: Locale,
    ) -> Vec<CompilerHash>,
    /// Data compilation function.
    #[allow(clippy::type_complexity)]
    pub compile_func: fn(
        source: ResourceId,
        dependencies: &[ResourceId],
        target: Target,
        platform: Platform,
        locale: &Locale,
        compiled_asset_store_path: CompiledAssetStoreAddr,
        resource_dir: &Path, // todo: assume sources are in the same directory? or cwd? or make this the resource dir?
    ) -> Result<Vec<CompiledAsset>, CompilerError>,
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
    /// Unknown platform.
    InvalidPlatform,
    /// Unknown target.
    InvalidTarget,
    /// Unknown command.
    UnknownCommand,
    /// Asset read/write failure.
    AssetStoreError,
    /// IO failure.
    ResourceLoadFailed(io::Error),
}

impl std::error::Error for CompilerError {}
impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CompilerError::StdoutError => write!(f, "IOError"),
            CompilerError::InvalidArgs => write!(f, "InvalidArgs"),
            CompilerError::InvalidResourceId => write!(f, "InvalidResourceId"),
            CompilerError::InvalidTarget => write!(f, "InvalidTarget"),
            CompilerError::InvalidPlatform => write!(f, "InvalidPlatform"),
            CompilerError::UnknownCommand => write!(f, "UnknownCommand"),
            CompilerError::AssetStoreError => write!(f, "AssetStoreError"),
            CompilerError::ResourceLoadFailed(_) => write!(f, "ResourceLoadFailed"),
        }
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
                    Arg::with_name(COMMAND_ARG_RESOURCE)
                        .required(true)
                        .help("Source to compile."),
                )
                .arg(
                    Arg::with_name(COMMAND_ARG_DEPENDENCIES)
                        .takes_value(true)
                        .long(COMMAND_ARG_DEPENDENCIES)
                        .multiple(true)
                        .help("Source dependencies to include."),
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
                        .help("Compiled Asset Store addresses where assets will be output."),
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

            let compiler_hash_list = (descriptor.compiler_hash_func)(
                descriptor.code_version,
                descriptor.data_version,
                target,
                platform,
                locale,
            );
            let output = CompilerHashCmdOutput { compiler_hash_list };
            serde_json::to_writer_pretty(stdout(), &output)
                .map_err(|_e| CompilerError::StdoutError)?;
            Ok(())
        }
        (COMMAND_NAME_COMPILE, Some(cmd_args)) => {
            let source = cmd_args.value_of(COMMAND_ARG_RESOURCE).unwrap();
            let target = cmd_args.value_of(COMMAND_ARG_TARGET).unwrap();
            let platform = cmd_args.value_of(COMMAND_ARG_PLATFORM).unwrap();
            let locale = cmd_args.value_of(COMMAND_ARG_LOCALE).unwrap();

            let source =
                ResourceId::from_str(source).map_err(|_e| CompilerError::InvalidResourceId)?;
            let target = Target::from_str(target).map_err(|_e| CompilerError::InvalidPlatform)?;
            let platform =
                Platform::from_str(platform).map_err(|_e| CompilerError::InvalidPlatform)?;
            let locale = Locale::new(locale);

            let deps: Vec<ResourceId> = cmd_args
                .values_of(COMMAND_ARG_DEPENDENCIES)
                .unwrap_or_default()
                .filter_map(|s| ResourceId::from_str(s).ok())
                .collect();
            let asset_store_path = CompiledAssetStoreAddr::from(
                cmd_args.value_of(COMMAND_ARG_COMPILED_ASSET_STORE).unwrap(),
            );
            let resource_dir = PathBuf::from(cmd_args.value_of(COMMAND_ARG_RESOURCE_DIR).unwrap());

            let compiled_assets = (descriptor.compile_func)(
                source,
                &deps,
                target,
                platform,
                &locale,
                asset_store_path,
                &resource_dir,
            )?;

            let output = CompilerCompileCmdOutput { compiled_assets };
            serde_json::to_writer_pretty(stdout(), &output)
                .map_err(|_e| CompilerError::StdoutError)?;
            Ok(())
        }
        _ => Err(CompilerError::UnknownCommand),
    }
}

fn resource_path(dir: &Path, id: ResourceId) -> PathBuf {
    let mut path = dir.to_owned();
    path.push(format!("{:x}", id));
    path.set_extension(RESOURCE_EXT);
    path
}

/// Synchronously loades a resource from current working directory.
pub fn compiler_load_resource(
    id: ResourceId,
    dir: impl AsRef<Path>,
    resources: &mut ResourceRegistry,
) -> Result<ResourceHandle, CompilerError> {
    let resource_path = resource_path(dir.as_ref(), id);
    let mut resource_file = File::open(resource_path).map_err(CompilerError::ResourceLoadFailed)?;
    let handle = resources
        .deserialize_resource(id.resource_type(), &mut resource_file)
        .map_err(CompilerError::ResourceLoadFailed)?;
    Ok(handle)
}
