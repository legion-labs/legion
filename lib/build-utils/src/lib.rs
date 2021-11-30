//! Legion build utils
//!

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

use bitflags::bitflags;
use std::ffi::OsStr;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Io error `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Failed to build with the following error `{0}`")]
    Build(String),
    #[error("Missing necessary tool `{0}`")]
    MissingTool(#[from] which::Error),
    #[error("unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;

bitflags! {
    pub struct Language : u32 {
        const RUST = 0;
        const TYPESCRIPT = 1;
    }
}

pub struct Context {
    crate_out_dir: String,
    codegen_dir: String,
    validate: bool,
}

impl Context {
    pub fn new(validate: bool) -> Self {
        Self {
            crate_out_dir: std::env::var("OUT_DIR").unwrap(),
            codegen_dir: String::from("./codegen"),
            validate,
        }
    }
}

/// Build proto files
///
/// # Errors
/// Returns a generation error or an IO error
///
#[cfg(feature = "proto-codegen")]
pub fn build_protos(
    context: &Context,
    protos: &[impl AsRef<Path>],
    includes: &[impl AsRef<Path>],
    lang: Language,
) -> Result<()> {
    let out_dir = PathBuf::from(&context.crate_out_dir);

    if lang.contains(Language::RUST) {
        tonic_build::configure()
            .out_dir(&out_dir)
            .compile(protos, includes)?;
    }
    if lang.contains(Language::TYPESCRIPT) {
        if Path::new("./package.json").exists() {
            run_cmd("yarn", &["install"], ".")?;

            let mut proto_plugin = PathBuf::from("./node_modules/.bin/protoc-gen-ts_proto");
            if cfg!(windows) {
                proto_plugin = PathBuf::from(".\\node_modules\\.bin\\protoc-gen-ts_proto.cmd");
            }
            if !proto_plugin.exists() {
                return Err(Error::Build(
                    "missing `ts-proto` in your package dependency".to_string(),
                ));
            }
            let plugin_arg = format!("--plugin=protoc-gen-ts_proto={}", proto_plugin.display());
            let proto_out_arg = format!("--ts_proto_out={}", out_dir.display());
            let mut args = vec![
                plugin_arg.as_str(),
                proto_out_arg.as_str(),
                "--ts_proto_opt=esModuleInterop=true",
                "--ts_proto_opt=outputClientImpl=grpc-web",
                "--ts_proto_opt=env=browser",
                "--ts_proto_opt=lowerCaseServiceMethods=true",
            ];
            let includes: Vec<_> = includes
                .iter()
                .map(|path| format!("--proto_path={}", path.as_ref().to_str().unwrap()))
                .collect();
            let mut include_args: Vec<_> =
                includes.iter().map(std::string::String::as_str).collect();
            args.append(&mut include_args);

            let mut protos_args: Vec<_> = protos
                .iter()
                .map(|path| path.as_ref().to_str().unwrap())
                .collect();
            args.append(&mut protos_args);
            run_cmd("protoc", &args, ".")?;
        } else {
            return Err(Error::Build(
                "a package.json file needs to be next to the build.rs".to_string(),
            ));
        }
    }

    for proto in protos {
        println!("cargo:rerun-if-changed={}", proto.as_ref().display());
    }

    Ok(())
}

/// Build proto files
///
/// # Errors
/// Returns a generation error or an IO error
///
pub fn handle_output(context: &Context) -> Result<()> {
    let diffs = diff(&context.crate_out_dir, &context.codegen_dir)?;
    for diff in &diffs {
        if context.validate {
            println!("cargo:warning={}", diff);
        } else {
            diff.apply()?;
        }
    }
    if context.validate && !diffs.is_empty() {
        Err(Error::Build(format!(
            "Generated files different from source (number of diffs: {})",
            diffs.len()
        )))
    } else {
        Ok(())
    }
}

fn run_cmd<S: AsRef<OsStr>>(command_path: S, args: &[&str], dir: &str) -> Result<()> {
    let command_path = which::which(command_path)?;
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
