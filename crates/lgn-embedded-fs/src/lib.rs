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

// crate-specific lint exceptions:
//#![allow()]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bus::{Bus, BusReader};
use linkme::distributed_slice;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};

pub mod macros;

#[distributed_slice]
pub static EMBEDDED_FILES: [EmbeddedFile] = [..];

pub struct EmbeddedFile {
    path: &'static str,
    content: &'static [u8],
    original_path: Option<&'static str>,
}

impl EmbeddedFile {
    pub const fn new(
        path: &'static str,
        content: &'static [u8],
        original_path: Option<&'static str>,
    ) -> Self {
        Self {
            path,
            content,
            original_path,
        }
    }
    pub const fn path(&'static self) -> &'static str {
        self.path
    }

    pub const fn content(&'static self) -> &'static [u8] {
        self.content
    }

    pub const fn original_path(&'static self) -> Option<&'static str> {
        self.original_path
    }
}

pub struct EmbeddedFileSystem {
    path_to_content: HashMap<&'static str, &'static EmbeddedFile>,
    bus: Arc<Mutex<Bus<&'static str>>>,
}

impl EmbeddedFileSystem {
    /// Initializes the embedded file system.
    pub fn init() -> Self {
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.
        let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

        let mut path_to_content = HashMap::<&'static str, &'static EmbeddedFile>::new();
        let mut watched_to_path = HashMap::<PathBuf, &'static str>::new();
        for file in EMBEDDED_FILES {
            println!("{} -- {:?}", file.path, file.original_path);
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

    /// Returns the origin path if available
    ///
    /// # Errors
    /// If the file is not found
    ///
    pub fn original_path<P: AsRef<Path>>(&self, path: P) -> Result<Option<&Path>, std::io::Error> {
        let path = path.as_ref();
        if let Some(file) = self.path_to_content.get(path.to_str().unwrap()) {
            Ok(file.original_path.map(Path::new))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.to_str().unwrap()),
            ))
        }
    }

    /// Returns the content of the file
    ///
    /// This content returns the most up to date content of the file meaning not necessarily
    /// the content it was compiled with if an original path exists
    ///
    /// # Errors
    /// If the file is not found
    ///
    pub fn read_all<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>, std::io::Error> {
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

    /// Returns the content of the file as a string
    ///
    /// # Errors
    /// If the file is not found
    ///
    pub fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String, std::io::Error> {
        let content = self.read_all(path)?;
        Ok(String::from_utf8_lossy(&content).to_string())
    }

    /// Add a receiver to the watch bus
    pub fn add_receiver(&mut self) -> BusReader<&'static str> {
        self.bus.lock().unwrap().add_rx()
    }
}

#[cfg(test)]
mod tests {
    use crate::{embedded_file, EmbeddedFileSystem};

    embedded_file!(FILE, "tests/data/test.txt");

    #[test]
    fn test_embedded_file_macro() {
        assert_eq!(FILE.path(), "crate://lgn-embedded-fs/tests/data/test.txt");
        assert_eq!(FILE.content(), b"Hello World!");
    }

    #[test]
    fn test_embedded_fs() {
        let embedded_fs = EmbeddedFileSystem::init();
        assert_eq!(FILE.content(), embedded_fs.read_all(FILE.path()).unwrap());
    }
}
