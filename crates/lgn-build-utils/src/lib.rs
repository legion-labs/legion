//! Legion build utils
//! This crate is meant to provide helpers for code generation in the monorepo
//! We rely on code generation in multiple instances:
//! * Proto files that generate rust and javascript files
//! * Shader files definition that generate rust and hlsl
//! * Data containers that generate rust files
//!

// crate-specific lint exceptions:
//#![allow()]

use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Io error `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Failed to build with the following error `{0}`")]
    MissingVar(#[from] std::env::VarError),
    #[error("Failed to build with the following error `{0}`")]
    Build(String),
    #[error("unknown error")]
    Unknown,
}

const OUT_DIR_SYMLINK_VAR: &str = "LGN_SYMLINK_OUT_DIR";

pub type Result<T> = std::result::Result<T, Error>;

/// Creates a temporary symlink to the `out_dir` next to the crate sources
/// This a helper to debug the generated files
/// It is not meant to be used in production
/// # Errors
/// Errors on missing environment variables set by the build system
/// Errors on symlink creation
///
pub fn symlink_out_dir() -> Result<()> {
    let manifest_out_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR")?).join("out_dir");
    let out_dir = PathBuf::from(&std::env::var("OUT_DIR")?);
    println!("cargo:rerun-if-env-changed={}", OUT_DIR_SYMLINK_VAR);
    if let Ok(value) = std::env::var(OUT_DIR_SYMLINK_VAR) {
        let mut create_symlink = value == "1" || value == "true";
        if let Ok(attr) = std::fs::symlink_metadata(&manifest_out_dir) {
            create_symlink = attr.is_symlink()
                && remove_symlink_dir(&manifest_out_dir).is_ok()
                && create_symlink;
        }
        if create_symlink {
            return create_symlink_dir(&out_dir, &manifest_out_dir)
                .map_err(std::convert::Into::into);
        }
    }
    Ok(())
}

fn create_symlink_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    return std::os::windows::fs::symlink_dir(src, dst);
    #[cfg(not(windows))]
    return std::os::unix::fs::symlink(src, dst);
}

fn remove_symlink_dir(src: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    return std::fs::remove_dir(src);
    #[cfg(not(windows))]
    return std::fs::remove_file(src);
}
