use std::{path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use hassle_rs::DxcIncludeHandler;
use lgn_embedded_fs::EmbeddedFileSystem;
use normpath::BasePathBuf;

struct FileSystemInner {
    embedded_fs: EmbeddedFileSystem,
    root_path: BasePathBuf,
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
    pub fn new(root_folder: &str) -> Result<Self> {
        let root_path = BasePathBuf::new(Path::new(root_folder)).unwrap();
        let root_path = root_path.normalize().unwrap();
        if !root_path.is_absolute() {
            return Err(anyhow!(
                "Root folder must refer to an absolute path ({})",
                root_folder
            ));
        }

        if !root_path.is_dir() {
            return Err(anyhow!(
                "Root folder must refer to a directory ({})",
                root_folder
            ));
        }

        Ok(Self {
            inner: Arc::new(FileSystemInner {
                embedded_fs: EmbeddedFileSystem::init(),
                root_path,
            }),
        })
    }

    /// Removes a mount point from the file system.
    ///
    /// # Errors
    /// fails if the mount point does not exist.
    ///
    pub fn translate_path(&self, path: &str) -> Result<BasePathBuf> {
        let protocol = "crate://";
        if path.starts_with(protocol) {
            return self.inner.embedded_fs.original_path(path)?.map_or_else(
                || Err(anyhow!("No original path")),
                |path| Ok(BasePathBuf::new(path).unwrap()),
            );
        }
        // this is not tested
        let path = self.inner.root_path.join(path);
        path.normalize()
            .map_err(|_err| anyhow!("Path is not valid"))
    }

    /// Translates a path to a file name.
    ///
    /// # Errors
    /// fails if the path does not exist.
    ///
    pub fn read_to_string(&self, path: &str) -> Result<String> {
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
