use std::sync::Arc;

use anyhow::{anyhow, Result};
use hassle_rs::DxcIncludeHandler;
use lgn_embedded_fs::EmbeddedFileSystem;
use normpath::BasePathBuf;

struct FileSystemInner {
    embedded_fs: EmbeddedFileSystem,
}

#[derive(Clone)]
pub struct FileSystem {
    inner: Arc<FileSystemInner>,
}

impl FileSystem {
    /// Creates a new file system.
    ///
    /// # Errors
    /// fails if the root path does not exist.
    ///
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(FileSystemInner {
                embedded_fs: EmbeddedFileSystem::init(),
            }),
        }
    }

    /// Removes a mount point from the file system.
    ///
    /// # Errors
    /// fails if the mount point does not exist.
    ///
    pub(crate) fn translate_path(&self, path: &str) -> Result<BasePathBuf> {
        self.inner.embedded_fs.original_path(path)?.map_or_else(
            || Err(anyhow!("No original path")),
            |path| Ok(BasePathBuf::new(path).unwrap()),
        )
    }

    /// Translates a path to a file name.
    ///
    /// # Errors
    /// fails if the path does not exist.
    ///
    pub(crate) fn read_to_string(&self, path: &str) -> Result<String> {
        let protocol = "crate://";
        if path.starts_with(protocol) {
            self.inner
                .embedded_fs
                .read_to_string(&path)
                .map_err(|err| anyhow!("{}", err))
        } else {
            return Err(anyhow!("Invalid path"));
        }
    }
}

pub struct FileServerIncludeHandler(pub FileSystem); // stack

impl DxcIncludeHandler for FileServerIncludeHandler {
    fn load_source(&mut self, file_name: String) -> Option<String> {
        // The compiler append "./" to the file name, we need to remove it
        let fixed_up_path = if let Some(pos) = file_name.find("crate://") {
            &file_name[pos..]
        } else {
            &file_name[..]
        };
        self.0.read_to_string(fixed_up_path).ok()
    }
}
