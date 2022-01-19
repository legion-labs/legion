//! Legion build utils
//! This crate is meant to provide helpers for code generation in the monorepo
//! We rely on code generation in multiple instances:
//! * Proto files that generate rust and javascript files
//! * Shader files definition that generate rust and hlsl
//! * Data containers that generate rust files
//!
//! There is 2 ways of handling generated files in the rust ecosystem:
//! * Relying on `OUT_DIR` environment variable to generate in place any
//!   necessary file. (tonic, windows api, ...)
//! * Generating the files in the repo and committing them to the repo.
//!   (rust-analyser, rusoto, ...)
//!
//! We can't generate files in the crate directory and not have them committed,
//! since we have to think about the case of an external dependency being
//! downloaded in the local immutable register.
//!
//! Advantages:
//! * Improves readability and UX of generated files (Go to definition works in
//!   VS Code, looking at code from github)
//! * Allows inclusion of generated files from other systems (Javasctript, hlsl
//!   in a uniform manner) since `OUT_DIR` is only know during the cargo build
//!   of a given crate.
//!
//! Drawbacks:
//! * Dummy conflict in generated code
//! * We lose the ability to modify some src files from the github web interface
//!   since you
//! * Confusion about non generated code and generated code (although mitigated
//!   by conventions)
//!
//! Restriction and rules:
//! * We can't have binary files checked in
//! * Modification of the generated files would not be allowed under any
//!   circumstances, the build machines fail if any change was detected
//! * Files whose generation ca be driven by features, or that are platform
//!   dependent would still use `OUT_DIR`.
//! * Other cases where the in repo generation doesn't bring much

// crate-specific lint exceptions:
//#![allow()]

use std::{
    ffi::OsStr,
    fmt::Formatter,
    path::{Path, PathBuf},
    process::Command,
};

use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Io error `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Failed to build with the following error `{0}`")]
    Build(String),
    #[error("Missing necessary tool `{0}`")]
    MissingTool(String),
    #[error("unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Context {
    codegen_out_dir: PathBuf,
    codegen_repo_dir: PathBuf,
    validation_only: bool,
}

impl Context {
    fn new(validation_only: bool) -> Self {
        Self {
            codegen_out_dir: Path::new(&std::env::var("OUT_DIR").unwrap()).join("codegen"),
            codegen_repo_dir: PathBuf::from("./codegen"),
            validation_only,
        }
    }

    pub fn codegen_out_dir(&self) -> &Path {
        &self.codegen_out_dir
    }
}

/// run commands in the shell
///
/// # Errors
/// Returns a `Error::MissingTool` if the command is not found
/// Returns a `Error::Build` if the command fails
pub fn run_cmd<S: AsRef<OsStr>>(command_path: S, args: &[&str], dir: &str) -> Result<()> {
    let command_path = which::which(command_path.as_ref())
        .map_err(|_err| Error::MissingTool(command_path.as_ref().to_str().unwrap().to_string()))?;
    let success = Command::new(&command_path)
        .args(args)
        .current_dir(dir)
        .status()?
        .success();

    if success {
        Ok(())
    } else {
        Err(Error::Build(format!(
            "Failed to run command {} with {} in {}",
            command_path.display(),
            args.join(" "),
            dir
        )))
    }
}

/// Creates a generation context, and cleans the temporary output folder
///
/// # Errors
/// Returns a generation error or an IO error
pub fn pre_codegen(validation_mode: bool) -> Result<Context> {
    let context = Context::new(validation_mode);
    if context.codegen_out_dir.exists() {
        std::fs::remove_dir_all(&context.codegen_out_dir)?;
    }
    std::fs::create_dir_all(&context.codegen_out_dir)?;

    Ok(context)
}

/// Handle the copy/validation of the output files
///
/// # Errors
/// Returns a generation error or an IO error
pub fn post_codegen(context: &Context) -> Result<()> {
    let diffs = diff(&context.codegen_out_dir, &context.codegen_repo_dir)?;
    for diff in &diffs {
        if context.validation_only {
            println!("cargo:warning={}", diff);
        } else {
            diff.apply()?;
        }
    }
    if context.validation_only && !diffs.is_empty() {
        Err(Error::Build(format!(
            "Generated files different from source (number of diffs: {})",
            diffs.len()
        )))
    } else {
        Ok(())
    }
}

enum Diff {
    Modified(PathBuf, Vec<u8>),
    Added(PathBuf, Vec<u8>),
    Deleted(PathBuf),
}

impl Diff {
    fn apply(&self) -> Result<()> {
        match self {
            Diff::Modified(path, content) => {
                std::fs::write(path, content)?;
            }
            Diff::Added(path, content) => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, content)?;
            }
            Diff::Deleted(a) => std::fs::remove_file(a)?,
        }
        Ok(())
    }
}

impl std::fmt::Display for Diff {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Diff::Modified(path, _) => f.write_fmt(format_args!("Modified: {}", path.display())),
            Diff::Added(path, _) => f.write_fmt(format_args!("Added: {}", path.display())),
            Diff::Deleted(path) => f.write_fmt(format_args!("Deleted: {}", path.display())),
        }
    }
}

fn diff<A: AsRef<Path>, B: AsRef<Path>>(source: A, destination: B) -> Result<Vec<Diff>> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    let mut diffs = walk_diff(source, destination, false)?;
    diffs.append(&mut walk_diff(destination, source, true)?);

    Ok(diffs)
}

fn walk_diff(source: &Path, destination: &Path, deletion_mode: bool) -> Result<Vec<Diff>> {
    let mut diffs = vec![];
    let mut source_walker = WalkDir::new(source).into_iter();
    loop {
        let source_entry = match source_walker.next() {
            None => break,
            Some(Err(_)) => continue,
            Some(Ok(entry)) => entry,
        };

        if !source_entry.file_type().is_file() {
            continue;
        }

        if source_entry.path_is_symlink() {
            continue;
        }

        let source_path_without_prefix = source_entry.path().strip_prefix(source).unwrap();
        let destination_path = destination.join(source_path_without_prefix);
        if deletion_mode {
            if !destination_path.is_file() {
                diffs.push(Diff::Deleted(source_entry.path().to_path_buf()));
            }
            continue;
        }
        let source_content = std::fs::read(source_entry.path())?;
        if !destination_path.is_file() {
            diffs.push(Diff::Added(
                destination_path,
                std::fs::read(source_entry.path())?,
            ));
            continue;
        }
        if source_content != std::fs::read(&destination_path)? {
            diffs.push(Diff::Modified(destination_path, source_content));
        }
    }
    Ok(diffs)
}
