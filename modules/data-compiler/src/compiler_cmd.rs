//! Interface to interact with data compilers.
//!
//! Data compiler is a binary that takes as input [`legion_resources::Resource`] and produces [`legion_assets::Asset`]s.
//! Because each *data compiler* is an external binary interacting with them can be challenging.
//!
//! [`compiler_cmd`] provides utilities that simplify interactions with data compilers.
//!
//! # Examples
//!
//! You can retrieve information about all compilers in a specified directory:
//!
//! ```
//! # use legion_data_compiler::compiler_cmd::{list_compilers, CompilerInfoCmd};
//! # use std::slice;
//! # use std::path::PathBuf;
//! for compiler in list_compilers(slice::from_ref(&PathBuf::from("./compilers/"))) {
//!     let command = CompilerInfoCmd::default();
//!     let info = command.execute(&compiler.path).expect("info output");
//! }
//! ```
//!
//! Retrieve the information about a specific compiler:
//!
//! ```no_run
//! # use legion_data_compiler::compiler_cmd::CompilerHashCmd;
//! # use legion_data_compiler::{Locale, Platform, Target};
//! let command = CompilerHashCmd::new(Target::Game, Platform::Windows, &Locale::new("en"));
//! let info = command.execute("my_compiler.exe").expect("compiler hash info");
//! ```
//!
//! Or compile the resources:
//!
//! ```no_run
//! # use legion_data_compiler::compiler_cmd::CompilerCompileCmd;
//! # use legion_data_compiler::compiled_asset_store::CompiledAssetStoreAddr;
//! # use legion_data_compiler::{Locale, Platform, Target};
//! # use legion_resources::ResourceId;
//! # use std::path::PathBuf;
//! fn compile_resource(id: ResourceId, dependencies: &[ResourceId]) {
//!     let asset_store = CompiledAssetStoreAddr::from("./asset_store/");
//!     let resource_dir = PathBuf::from("./resources/");
//!     let mut command = CompilerCompileCmd::new(id, dependencies, &asset_store, &resource_dir, Target::Game, Platform::Windows, &Locale::new("en"));
//!     let output = command.execute("my_compiler.exe", "./").expect("compiled assets");
//!}
//! ```
//!
//! For more about data compilers see [`compiler_api`] module.
//!
//! [`compiler_api`]: ../compiler_api/index.html
//! [`compiler_cmd`]: ./index.html

use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{
    compiler_api::CompilerDescriptor, CompiledAsset, CompiledAssetStoreAddr, CompilerHash, Locale,
    Platform, Target,
};

use legion_resources::{ResourceId, ResourceType};

use serde::{Deserialize, Serialize};

/// Description of a compiler.
#[derive(Debug, Clone)]
pub struct CompilerInfo {
    /// Name of the compiler.
    pub name: String,
    /// Binary location.
    pub path: PathBuf,
}

/// Returns a list of compilers found at locations `paths`.
pub fn list_compilers(paths: &[PathBuf]) -> Vec<CompilerInfo> {
    let mut commands = Vec::new();
    let prefix = "compiler-";
    let suffix = env::consts::EXE_SUFFIX;
    for dir in search_directories(paths) {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let filename = match path.file_name().and_then(|s| s.to_str()) {
                    Some(filename) => filename,
                    _ => continue,
                };
                if !filename.starts_with(prefix) || !filename.ends_with(suffix) {
                    continue;
                }
                if !is_executable(&path) {
                    continue;
                }
                let name = filename[prefix.len()..(filename.len() - suffix.len())].to_string();
                commands.push(CompilerInfo {
                    name,
                    path: path.clone(),
                });
            }
        }
    }

    commands
}

fn search_directories(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs = paths.to_owned();
    if let Ok(cwd) = env::current_dir() {
        dirs.push(cwd);
    }
    dirs
}

#[cfg(unix)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

struct CommandBuilder {
    args: Vec<String>,
    cwd: Option<PathBuf>,
}

impl CommandBuilder {
    /// Creates a new [`CommandBuilder`] with the given executable path.
    fn default() -> Self {
        Self {
            args: vec![],
            cwd: None,
        }
    }

    /// Adds `arg` to the args list.
    fn arg<T: Into<String>>(&mut self, arg: T) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    /// Sets current working directory for the process.
    fn cwd<T: AsRef<Path>>(&mut self, cwd: T) -> &mut Self {
        self.cwd = Some(cwd.as_ref().to_owned());
        self
    }

    /// Executes the process returing the stdio output or an error on non-zero exit status.
    fn exec<T: AsRef<OsStr>>(&self, compiler_path: T) -> io::Result<std::process::Output> {
        let mut command = std::process::Command::new(compiler_path);
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        command.args(&self.args);

        let output = command.output()?;

        if output.status.success() {
            Ok(output)
        } else {
            println!("{}", String::from_utf8(output.stdout).expect("valid utf8"));
            println!("{}", String::from_utf8(output.stderr).expect("valid utf8"));
            Err(io::Error::new(io::ErrorKind::Other, "Status Error"))
        }
    }
}

//
// Compiler Info Command
//

#[derive(Serialize, Deserialize, Debug)]
/// Output of `compiler_info` command.
pub struct CompilerInfoCmdOutput {
    /// Code version of data compiler.
    pub code_version: String,
    /// Resource and Asset data version.
    pub data_version: String,
    /// Resource types supported by data compiler.
    pub resource_type: Vec<ResourceType>,
}

impl CompilerInfoCmdOutput {
    pub(crate) fn from_descriptor(descriptor: &CompilerDescriptor) -> Self {
        Self {
            code_version: descriptor.code_version.to_owned(),
            data_version: descriptor.data_version.to_owned(),
            resource_type: descriptor.resource_types.to_owned(),
        }
    }
    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()?
    }
}

pub(crate) const COMMAND_NAME_INFO: &str = "info";
pub(crate) const COMMAND_NAME_COMPILER_HASH: &str = "compiler_hash";
pub(crate) const COMMAND_NAME_COMPILE: &str = "compile";
pub(crate) const COMMAND_ARG_PLATFORM: &str = "platform";
pub(crate) const COMMAND_ARG_TARGET: &str = "target";
pub(crate) const COMMAND_ARG_LOCALE: &str = "locale";
pub(crate) const COMMAND_ARG_RESOURCE: &str = "resource";
pub(crate) const COMMAND_ARG_DEPENDENCIES: &str = "deps";
pub(crate) const COMMAND_ARG_COMPILED_ASSET_STORE: &str = "cas";
pub(crate) const COMMAND_ARG_RESOURCE_DIR: &str = "resource_dir";

/// Helper building a `info` command.
pub struct CompilerInfoCmd(CommandBuilder);

impl CompilerInfoCmd {
    /// Creates a new command.
    pub fn default() -> Self {
        let mut builder = CommandBuilder::default();
        builder.arg(COMMAND_NAME_INFO);
        Self(builder)
    }

    /// Runs the command on compiler process located at `compiler_path`, waits for completion, returns the result.
    pub fn execute(&self, compiler_path: impl AsRef<OsStr>) -> io::Result<CompilerInfoCmdOutput> {
        let output = self.0.exec(compiler_path)?;
        CompilerInfoCmdOutput::from_bytes(output.stdout.as_slice()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse CompilerInfoCmdOutput",
            )
        })
    }
}

//
// Compiler Hash Command
//

/// Output of `compiler_hash` command.
#[derive(Serialize, Deserialize, Debug)]
pub struct CompilerHashCmdOutput {
    /// List of [`CompilerHash`] values for given compilation context.
    pub compiler_hash_list: Vec<CompilerHash>,
}

impl CompilerHashCmdOutput {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()?
    }
}

/// Helper building a `compiler_hash` command.
pub struct CompilerHashCmd(CommandBuilder);

impl CompilerHashCmd {
    /// Creates a new command for a given compilation context.
    pub fn new(target: Target, platform: Platform, locale: &Locale) -> Self {
        let mut builder = CommandBuilder::default();
        builder.arg(COMMAND_NAME_COMPILER_HASH);
        builder.arg(format!("--{}={}", COMMAND_ARG_TARGET, target));
        builder.arg(format!("--{}={}", COMMAND_ARG_PLATFORM, platform));
        builder.arg(format!("--{}={}", COMMAND_ARG_LOCALE, locale.0));
        Self(builder)
    }

    /// Runs the command on compiler process located at `compiler_path`, waits for completion, returns the result.
    pub fn execute(&self, compiler_path: impl AsRef<OsStr>) -> io::Result<CompilerHashCmdOutput> {
        let output = self.0.exec(compiler_path)?;
        CompilerHashCmdOutput::from_bytes(output.stdout.as_slice()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse CompilerHashCmdOutput",
            )
        })
    }
}

//
// Compiler Compile Command
//

/// Output of `compile` command.
#[derive(Serialize, Deserialize, Debug)]
pub struct CompilerCompileCmdOutput {
    /// Generated assets.
    pub compiled_assets: Vec<CompiledAsset>,
}

impl CompilerCompileCmdOutput {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()?
    }
}

/// Helper building a `compile` command.
pub struct CompilerCompileCmd(CommandBuilder);

impl CompilerCompileCmd {
    /// Creates a new command.
    pub fn new(
        source: ResourceId,
        deps: &[ResourceId],
        cas_addr: &CompiledAssetStoreAddr,
        resource_dir: &Path,
        target: Target,
        platform: Platform,
        locale: &Locale,
    ) -> Self {
        let mut builder = CommandBuilder::default();
        builder.arg(COMMAND_NAME_COMPILE);
        builder.arg(format!("{}", source));
        if !deps.is_empty() {
            let mut deps_arg = format!("--{}={}", COMMAND_ARG_DEPENDENCIES, deps[0]);
            for res in deps.iter().skip(1) {
                deps_arg.push_str(&format!(",{}", res));
            }
            builder.arg(deps_arg);
        }

        builder.arg(format!(
            "--{}={}",
            COMMAND_ARG_COMPILED_ASSET_STORE, cas_addr
        ));
        builder.arg(format!(
            "--{}={}",
            COMMAND_ARG_RESOURCE_DIR,
            resource_dir.display()
        ));

        builder.arg(format!("--{}={}", COMMAND_ARG_TARGET, target));
        builder.arg(format!("--{}={}", COMMAND_ARG_PLATFORM, platform));
        builder.arg(format!("--{}={}", COMMAND_ARG_LOCALE, locale.0));
        Self(builder)
    }

    /// Runs the command on compiler process located at `compiler_path` setting the current working directory
    /// of the compiler to `cwd`, waits for completion, returns the result.
    pub fn execute(
        &mut self,
        compiler_path: impl AsRef<OsStr>,
        cwd: impl AsRef<Path>,
    ) -> io::Result<CompilerCompileCmdOutput> {
        let output = self.0.cwd(cwd).exec(compiler_path)?;
        CompilerCompileCmdOutput::from_bytes(output.stdout.as_slice()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse CompilerCompileCmdOutput",
            )
        })
    }
}
