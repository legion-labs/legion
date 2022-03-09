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
use std::path::Path;
use std::sync::RwLock;

use once_cell::sync::Lazy;

pub mod macros;
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

#[derive(Default)]
pub struct EmbeddedFileSystem {
    path_to_content: RwLock<HashMap<&'static str, &'static EmbeddedFile>>,
}

pub static EMBEDDED_FS: Lazy<EmbeddedFileSystem> = Lazy::new(|| EmbeddedFileSystem {
    path_to_content: RwLock::new(HashMap::new()),
});

impl EmbeddedFileSystem {
    /// Initializes the embedded file system.

    pub fn add_file(&self, file: &'static EmbeddedFile) {
        self.path_to_content
            .write()
            .unwrap()
            .insert(file.path, file);
    }

    /// Returns the origin path if available
    ///
    /// # Errors
    /// If the file is not found
    ///
    pub fn original_path<P: AsRef<Path>>(&self, path: P) -> Result<Option<&Path>, std::io::Error> {
        let path = path.as_ref();
        if let Some(file) = self
            .path_to_content
            .read()
            .unwrap()
            .get(path.to_str().unwrap())
        {
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
        if let Some(file) = self
            .path_to_content
            .read()
            .unwrap()
            .get(path.to_str().unwrap())
        {
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
        let embedded_fs = EmbeddedFileSystem::default();
        embedded_fs.add_file(&FILE);
        assert_eq!(FILE.content(), embedded_fs.read_all(FILE.path()).unwrap());
    }
}
