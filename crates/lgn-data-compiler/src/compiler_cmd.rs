//! Interface to interact with data compilers.
//!
//! Data compiler is a binary that takes as input
//! [`lgn_data_runtime::Resource`]s. Because each *data compiler* is an external
//! binary interacting with them can be challenging.
//!
//! [`compiler_cmd`] provides utilities that simplify interactions with data
//! compilers.
//!
//! # Examples
//!
//! You can retrieve information about all compilers in a specified directory:
//!
//! ```
//! # use lgn_data_compiler::compiler_cmd::{list_compilers, CompilerInfoCmd};
//! # use std::slice;
//! # use std::path::PathBuf;
//! for compiler in list_compilers(slice::from_ref(&PathBuf::from("./compilers/"))) {
//!     let command = CompilerInfoCmd::new(&compiler.path);
//!     let info = command.execute().expect("info output");
//! }
//! ```
//!
//! Retrieve the information about a specific compiler:
//!
//! ```no_run
//! # use lgn_data_compiler::compiler_cmd::CompilerHashCmd;
//! # use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
//! let command = CompilerHashCmd::new("my_compiler.exe", &CompilationEnv{ target: Target::Game, platform: Platform::Windows, locale: Locale::new("en") }, None);
//! let info = command.execute().expect("compiler hash info");
//! ```
//!
//! Or compile the resources:
//!
//! ```no_run
//! # use lgn_data_compiler::compiler_cmd::CompilerCompileCmd;
//! # use lgn_content_store::ContentStoreAddr;
//! # use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
//! # use lgn_data_offline::ResourcePathId;
//! # use std::path::PathBuf;
//! fn compile_resource(compile_path: ResourcePathId, dependencies: &[ResourcePathId], env: &CompilationEnv) {
//!     let content_store = ContentStoreAddr::from("./content_store/");
//!     let resource_dir = PathBuf::from("./resources/");
//!     let mut command = CompilerCompileCmd::new("my_compiler.exe", &compile_path, dependencies, &[], &content_store, &resource_dir, &env);
//!     let output = command.execute().expect("compiled resources");
//! }
//! ```
//!
//! For more about data compilers see [`compiler_api`] module.
//!
//! [`compiler_api`]: ../compiler_api/index.html
//! [`compiler_cmd`]: ./index.html

use std::{
    env,
    ffi::{OsStr, OsString},
    fmt, fs, io,
    path::{Path, PathBuf},
};

use lgn_content_store::ContentStoreAddr;
use lgn_data_offline::{ResourcePathId, Transform};
use serde::{Deserialize, Serialize};

use crate::{
    compiler_api::{CompilationEnv, CompilerError, CompilerInfo},
    compiler_node::CompilerRegistry,
    CompiledResource, CompilerHash,
};

/// Description of a compiler.
#[derive(Debug, Clone)]
pub struct CompilerLocation {
    /// Name of the compiler.
    pub name: String,
    /// Binary location.
    pub path: PathBuf,
}

/// Returns a list of compilers found at locations `paths`.
pub fn list_compilers(paths: &[impl AsRef<Path>]) -> Vec<CompilerLocation> {
    let mut commands = Vec::new();
    let prefix = "compiler-";
    let suffix = env::consts::EXE_SUFFIX;
    for dir in search_directories(paths) {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.filter_map(std::result::Result::ok) {
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
                commands.push(CompilerLocation {
                    name,
                    path: path.clone(),
                });
            }
        }
    }

    commands
}

fn search_directories(paths: &[impl AsRef<Path>]) -> Vec<PathBuf> {
    let mut dirs = paths
        .iter()
        .map(|a| a.as_ref().to_owned())
        .collect::<Vec<_>>();
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

/// Represents a command-line call along with its arguments.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CommandBuilder {
    command: String,
    args: Vec<String>,
}

impl CommandBuilder {
    /// Create a command from a .json string.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CompilerError> {
        serde_json::from_slice::<Self>(bytes).map_err(CompilerError::SerdeJson)
    }

    /// Serialize the command into a .json string.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    /// Convert the command into a `OsString` vector.
    pub fn to_os_args(&self) -> Vec<OsString> {
        self.args.iter().map(Into::into).collect()
    }

    /// Sets the command's executable path.
    fn set_command(&mut self, compiler_path: impl AsRef<OsStr>) -> &mut Self {
        self.command = compiler_path.as_ref().to_string_lossy().to_string();
        self
    }

    /// Adds `arg` to the args list.
    fn arg(&mut self, arg: &str) -> &mut Self {
        if !arg.is_empty() {
            self.args.push(arg.to_string());
        }
        self
    }

    fn arg2<T: fmt::Display>(&mut self, arg: &str, arg2: Option<T>) -> &mut Self {
        if !arg.is_empty() && arg2.is_some() {
            self.args.push(format!("--{}={}", arg, &arg2.unwrap()));
        }
        self
    }

    fn many_args<T>(&mut self, arg: &str, vec: T) -> &mut Self
    where
        T: IntoIterator,
        T::Item: fmt::Display,
    {
        let mut str_vec: Vec<String> = vec.into_iter().map(|a| a.to_string()).collect();
        if !str_vec.is_empty() {
            self.args.push(format!("--{}", arg));
            self.args.append(&mut str_vec);
        }
        self
    }

    /// Executes the process returning the stdio output or an error on non-zero
    /// exit status.
    fn exec(&self) -> io::Result<std::process::Output> {
        self.exec_with_cwd(env::current_dir().unwrap().as_path())
    }

    /// Executes the process returning the stdio output or an error on non-zero
    /// exit status.
    fn exec_with_cwd(&self, current_dir: impl AsRef<Path>) -> io::Result<std::process::Output> {
        let output = std::process::Command::new(&self.command)
            .current_dir(current_dir)
            .args(&self.args)
            .output()?;
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

/// Output of `compiler_info` command.
#[derive(Serialize, Deserialize, Debug)]
pub struct CompilerInfoCmdOutput(Vec<CompilerInfo>);

impl CompilerInfoCmdOutput {
    pub(crate) fn from_registry(compilers: &CompilerRegistry) -> Self {
        Self(compilers.infos().clone())
    }
    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()?
    }

    /// Converts the the output to a list of `CompilerInfo`.
    pub fn take(self) -> Vec<CompilerInfo> {
        self.0
    }
}

pub(crate) const COMMAND_NAME_INFO: &str = "info";
pub(crate) const COMMAND_NAME_COMPILER_HASH: &str = "compiler_hash";
pub(crate) const COMMAND_NAME_COMPILE: &str = "compile";
pub(crate) const COMMAND_ARG_PLATFORM: &str = "platform";
pub(crate) const COMMAND_ARG_TARGET: &str = "target";
pub(crate) const COMMAND_ARG_LOCALE: &str = "locale";
pub(crate) const COMMAND_ARG_SRC_DEPS: &str = "deps";
pub(crate) const COMMAND_ARG_DER_DEPS: &str = "derdeps";
pub(crate) const COMMAND_ARG_COMPILED_ASSET_STORE: &str = "cas";
pub(crate) const COMMAND_ARG_RESOURCE_DIR: &str = "resource_dir";
pub(crate) const COMMAND_ARG_TRANSFORM: &str = "transform";

/// Helper building a `info` command.
#[derive(Serialize, Deserialize)]
pub struct CompilerInfoCmd(CommandBuilder);

impl CompilerInfoCmd {
    /// Creates a new command.
    pub fn new(compiler_path: impl AsRef<OsStr>) -> Self {
        Self(
            CommandBuilder::default()
                .set_command(compiler_path)
                .arg(COMMAND_NAME_INFO)
                .clone(),
        )
    }

    /// Create a new command from a .json string.
    pub fn from_slice(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }

    /// Runs the command on compiler process located at `compiler_path`, waits
    /// for completion, returns the result.
    pub fn execute(&self) -> io::Result<CompilerInfoCmdOutput> {
        let output = self.0.exec()?;
        CompilerInfoCmdOutput::from_bytes(output.stdout.as_slice()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse CompilerInfoCmdOutput",
            )
        })
    }

    /// Extracts the command line builder.
    pub fn builder(&self) -> CommandBuilder {
        self.0.clone()
    }
}

//
// Compiler Hash Command
//

/// Output of `compiler_hash` command.
#[derive(Serialize, Deserialize, Debug)]
pub struct CompilerHashCmdOutput {
    /// List of [`Transform`] and [`CompilerHash`] for given compilation context supported by given compiler.
    pub compiler_hash_list: Vec<(Transform, CompilerHash)>,
}

impl CompilerHashCmdOutput {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()?
    }
}

/// Helper building a `compiler_hash` command.
#[derive(Serialize, Deserialize)]
pub struct CompilerHashCmd(CommandBuilder);

impl CompilerHashCmd {
    /// Creates a new command for a given compilation context that will return a hash for a specified `transform`.
    /// It returns hashes for all transforms if a `transform` argument is None.
    pub fn new(
        compiler_path: impl AsRef<OsStr>,
        env: &CompilationEnv,
        transform: Option<Transform>,
    ) -> Self {
        Self(
            CommandBuilder::default()
                .set_command(compiler_path)
                .arg(COMMAND_NAME_COMPILER_HASH)
                .arg2(COMMAND_ARG_TARGET, env.target.into())
                .arg2(COMMAND_ARG_PLATFORM, env.platform.into())
                .arg2(COMMAND_ARG_LOCALE, env.locale.clone().into())
                .arg2(COMMAND_ARG_TRANSFORM, transform)
                .clone(),
        )
    }

    /// Create a new command from a .json string.
    pub fn from_slice(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }

    /// Runs the command on compiler process located at `compiler_path`, waits
    /// for completion, returns the result.
    pub fn execute(&self) -> io::Result<CompilerHashCmdOutput> {
        let output = self.0.exec()?;
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

/// Helper building a `compile` command.
#[derive(Serialize, Deserialize)]
pub struct CompilerCompileCmd(CommandBuilder);

impl CompilerCompileCmd {
    /// Creates a new command.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        compiler_path: impl AsRef<OsStr>,
        resource_to_build: &ResourcePathId,
        source_deps: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        cas_addr: &ContentStoreAddr,
        resource_dir: &Path,
        env: &CompilationEnv,
    ) -> Self {
        Self(
            CommandBuilder::default()
                .set_command(compiler_path)
                .arg(COMMAND_NAME_COMPILE)
                .arg(&resource_to_build.to_string())
                .many_args(COMMAND_ARG_SRC_DEPS, source_deps.iter())
                .many_args(COMMAND_ARG_DER_DEPS, derived_deps.iter())
                .arg2(COMMAND_ARG_COMPILED_ASSET_STORE, cas_addr.into())
                .arg2(COMMAND_ARG_RESOURCE_DIR, resource_dir.display().into())
                .arg2(COMMAND_ARG_TARGET, env.target.into())
                .arg2(COMMAND_ARG_PLATFORM, env.platform.into())
                .arg2(COMMAND_ARG_LOCALE, env.locale.clone().into())
                .clone(),
        )
    }

    /// Create a new command from a .json string.
    pub fn from_slice(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }

    /// Runs the command on compiler process located at `compiler_path` setting
    /// the current working directory of the compiler to `cwd`, waits for
    /// completion, returns the result.
    pub fn execute(&self) -> io::Result<CompilerCompileCmdOutput> {
        self.execute_with_cwd(env::current_dir().unwrap())
    }

    /// Runs the command on compiler process located at `compiler_path` setting
    /// the current working directory of the compiler to `cwd`, waits for
    /// completion, returns the result.
    pub fn execute_with_cwd(
        &self,
        current_dir: impl AsRef<Path>,
    ) -> io::Result<CompilerCompileCmdOutput> {
        match self.0.exec_with_cwd(current_dir) {
            Ok(output) => CompilerCompileCmdOutput::from_bytes(&output.stdout).map_err(|_e| {
                eprintln!(
                    "Cannot parse compiler output, {:?} {:?}\nError: {:?}",
                    self.0.command, self.0.args, &output.stdout
                );
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Failed to parse CompilerCompileCmdOutput: `{}`",
                        std::str::from_utf8(output.stdout.as_slice()).unwrap()
                    ),
                )
            }),
            Err(e) => {
                eprintln!(
                    "Compiler command failed: {:?} {:?}",
                    self.0.command, self.0.args
                );
                Err(e)
            }
        }
    }
}
