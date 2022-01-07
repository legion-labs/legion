//! Legion Embedded FS
//! This crates exposes a file system allowing you use files embedded from
//! different crates
//!
//! Usage is as follow, to add files in shader-1 crate:
//! ```text
//! embedded_fs::watched_file!("cgen/def.hlsl");
//! embedded_fs::watched_file!("shader1.fx");
//! ```
//! And to consume them:
//! ```text
//! fn main() {
//!     let mut efs = EmbeddedFileSystem::init();
//!     let mut rx = efs.add_receiver();
//!     let path = rx.recv_timeout(std::time::Duration::from_secs(60)).unwrap();
//!     let a = efs.read("crates://shader-1/cgen/def.hlsl");
//!
//!     println!(
//!         "Dirty Path: {} -> {}",
//!         path,
//!         efs.read_as_string(path).unwrap()
//!     );
//! }
//! ```

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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bus::{Bus, BusReader};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};

pub struct EmbeddedFile {
    pub content: &'static [u8],
    pub path: &'static str,
    pub original_path: Option<&'static str>,
}

#[macro_export]
macro_rules! watched_file {
    ( $file_path:literal ) => {
        inventory::submit! {
            embedded_fs::EmbeddedFile{
                content: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file_path)),
                path: concat!("crate://", env!("CARGO_PKG_NAME"), "/", $file_path),
                original_path: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file_path)),
            }
        }
    };
}
#[macro_export]
macro_rules! file {
    ( $file_path:literal ) => {
        inventory::submit! {
            embedded_fs::EmbeddedFile{
                content: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file_path)),
                path: concat!(env!("CARGO_CRATE_NAME"), "/", $file_path),
                original_path: None,
            }
        }
    };
}

inventory::collect!(EmbeddedFile);

pub struct EmbeddedFileSystem {
    path_to_content: HashMap<&'static str, &'static EmbeddedFile>,
    bus: Arc<Mutex<Bus<&'static str>>>,
}

impl EmbeddedFileSystem {
    pub fn init() -> Self {
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.
        let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

        let mut path_to_content = HashMap::<&'static str, &'static EmbeddedFile>::new();
        let mut watched_to_path = HashMap::<PathBuf, &'static str>::new();
        for file in inventory::iter::<EmbeddedFile>() {
            println!("{}--{:?}", file.path, file.original_path);
            path_to_content.insert(file.path, file);
            if let Some(watch_path) = file.original_path {
                let watch_path = std::fs::canonicalize(watch_path).unwrap();
                watched_to_path.insert(watch_path.clone(), file.path);
                watcher
                    .watch(watch_path, RecursiveMode::NonRecursive)
                    .unwrap();
            }
        }

        let bus = Arc::new(Mutex::new(Bus::<&'static str>::new(10)));
        let bus_clone = bus.clone();
        std::thread::spawn(move || {
            let _watcher = watcher;
            loop {
                match rx.recv() {
                    Ok(DebouncedEvent::Write(ref path)) => {
                        println!("{:?}", path);
                        bus_clone
                            .lock()
                            .unwrap()
                            .broadcast(watched_to_path.get(path).unwrap());
                    }
                    Ok(event) => {
                        println!("{:?}", event);
                    }
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        });

        Self {
            path_to_content,
            bus,
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>, std::io::Error> {
        let path = path.as_ref();
        if let Some(file) = self.path_to_content.get(path.to_str().unwrap()) {
            if let Some(original_path) = file.original_path {
                if let Ok(content) = std::fs::read(original_path) {
                    return Ok(content);
                }
            }
            Ok(file.content.to_vec())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.to_str().unwrap()),
            ))
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn read_as_string<P: AsRef<Path>>(&self, path: P) -> Result<String, std::io::Error> {
        let content = self.read(path)?;
        Ok(String::from_utf8_lossy(&content).to_string())
    }

    pub fn add_receiver(&mut self) -> BusReader<&'static str> {
        self.bus.lock().unwrap().add_rx()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
