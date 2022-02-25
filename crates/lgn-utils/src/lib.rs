//! Legion core crate, contains core services and systems used by other modules
//! The crate is not allowed to depend on other legion modules

// crate-specific lint exceptions:
#![allow(clippy::implicit_hasher, clippy::missing_errors_doc)]

use std::{future::Future, io::Error, path::PathBuf, pin::Pin};

pub mod decimal;
pub mod memory;
pub mod trust_cell;

pub mod label;

mod hash;
pub use hash::*;

#[cfg(not(target_arch = "wasm32"))]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(target_arch = "wasm32")]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

fn find_path(mut root_dir: PathBuf) -> Option<String> {
    const MONOREPO_FILENAME: &str = "monorepo.toml";
    loop {
        let monorepo_file = root_dir.join(MONOREPO_FILENAME);
        if monorepo_file.is_file() {
            return Some(root_dir.to_string_lossy().to_string());
        }
        if !root_dir.pop() {
            return None;
        }
    }
}

pub fn find_monorepo_root() -> Result<String, Error> {
    let root_dir = std::env::current_dir().unwrap();
    if let Some(root_dir) = find_path(root_dir) {
        return Ok(root_dir);
    }
    let root_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    if let Some(root_dir) = find_path(root_dir) {
        return Ok(root_dir);
    }
    Err(Error::from(std::io::ErrorKind::NotFound))
}
