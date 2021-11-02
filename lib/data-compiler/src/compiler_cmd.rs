//! Interface to interact with data compilers.
//!
//! Data compiler is a binary that takes as input [`legion_data_runtime::Resource`]s.
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
//! # use legion_content_store::ContentStoreAddr;
//! # use legion_data_compiler::{Locale, Platform, Target};
//! # use legion_data_offline::ResourcePathId;
//! # use std::path::PathBuf;
//! fn compile_resource(compile_path: ResourcePathId, dependencies: &[ResourcePathId]) {
//!     let content_store = ContentStoreAddr::from("./content_store/");
//!     let resource_dir = PathBuf::from("./resources/");
//!     let mut command = CompilerCompileCmd::new(&compile_path, dependencies, &[], &content_store, &resource_dir, Target::Game, Platform::Windows, &Locale::new("en"));
//!     let output = command.execute("my_compiler.exe").expect("compiled resources");
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

use legion_content_store::ContentStoreAddr;
use legion_data_offline::ResourcePathId;
use legion_data_runtime::ResourceType;
use serde::{Deserialize, Serialize};

use crate::{
    compiler_api::CompilerDescriptor, CompiledResource, CompilerHash, Locale, Platform, Target,
};

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
                let filename = match path.file_name().and_then(OsStr::to_str) {
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
}

impl CommandBuilder {
    /// Creates a new [`CommandBuilder`] with the given executable path.
    fn default() -> Self {
        Self { args: vec![] }
    }

    /// Adds `arg` to the args list.
    fn arg<T: Into<String>>(&mut self, arg: T) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    /// Executes the process returning the stdio output or an error on non-zero exit status.
    fn exec<T: AsRef<OsStr>>(&self, compiler_path: T) -> io::Result<std::process::Output> {
        let mut command = std::process::Command::new(compiler_path);
        command.args(&self.args);

        let output = command.output()?;

        if output.status.success() {
            Ok(output)
        } else {
            println!("Process Exited With Code: {}", output.status);
            println!(
                "Stdout: '{}'",
                String::from_utf8(output.stdout).expect("valid utf8")
            );
            println!(
                "Stderr: '{}'",
                String::from_utf8(output.stderr).expect("valid utf8")
            );
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
    /// Data build version of data compiler.
    pub build_version: String,
    /// Code version of data compiler.
    pub code_version: String,
    /// Resource and Asset data version.
    pub data_version: String,
    /// Transformation supported by data compiler.
    pub transform: (ResourceType, ResourceType),
}

impl CompilerInfoCmdOutput {
    pub(crate) fn from_descriptor(descriptor: &CompilerDescriptor) -> Self {
        Self {
            build_version: descriptor.build_version.to_owned(),
            code_version: descriptor.code_version.to_owned(),
            data_version: descriptor.data_version.to_owned(),
            transform: descriptor.transform.to_owned(),
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
pub(crate) const COMMAND_ARG_RESOURCE_PATH: &str = "resource";
pub(crate) const COMMAND_ARG_SRC_DEPS: &str = "deps";
pub(crate) const COMMAND_ARG_DER_DEPS: &str = "derdeps";
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
    /// [`CompilerHash`] for given compilation context.
    pub compiler_hash: CompilerHash,
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
    /// Generated resources.
    pub compiled_resources: Vec<CompiledResource>,
    /// References between generated resources.
    pub resource_references: Vec<(ResourcePathId, ResourcePathId)>,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        compile_path: &ResourcePathId,
        source_deps: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        cas_addr: &ContentStoreAddr,
        resource_dir: &Path,
        target: Target,
        platform: Platform,
        locale: &Locale,
    ) -> Self {
        let mut builder = CommandBuilder::default();
        builder.arg(COMMAND_NAME_COMPILE);
        builder.arg(format!("{}", compile_path));
        if !source_deps.is_empty() {
            builder.arg(format!("--{}", COMMAND_ARG_SRC_DEPS));
            for res in source_deps {
                builder.arg(format!("{}", res));
            }
        }
        if !derived_deps.is_empty() {
            builder.arg(format!("--{}", COMMAND_ARG_DER_DEPS));
            for res in derived_deps {
                builder.arg(format!("{}", res));
            }
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
    ) -> io::Result<CompilerCompileCmdOutput> {
        match self.0.exec(compiler_path) {
            Ok(output) => CompilerCompileCmdOutput::from_bytes(output.stdout.as_slice())
                .ok_or_else(|| {
                    eprintln!("Cannot parse compiler output, args: {:?}", self.0.args);
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Failed to parse CompilerCompileCmdOutput: `{}`",
                            std::str::from_utf8(output.stdout.as_slice()).unwrap()
                        ),
                    )
                }),
            Err(e) => {
                eprintln!("Compiler command failed, args: {:?}", self.0.args);
                Err(e)
            }
        }
    }
}
