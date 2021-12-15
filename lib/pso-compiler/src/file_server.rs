use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};
use hassle_rs::DxcIncludeHandler;
use normpath::BasePathBuf;

struct MountPoint {
    name: String,
    path: BasePathBuf,
}

struct FileSystemInner {
    root_path: BasePathBuf,
    mount_points: RwLock<Vec<MountPoint>>,
}

#[derive(Clone)]
pub struct FileSystem {
    inner: Arc<FileSystemInner>,
}

impl FileSystem {
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
                root_path,
                mount_points: RwLock::new(Vec::new()),
            }),
        })
    }

    pub fn add_mount_point(&self, mount_point: &str, folder: &str) -> Result<()> {
        let path = BasePathBuf::new(Path::new(folder)).unwrap();
        let path = path.normalize().unwrap();
        if !path.is_absolute() {
            return Err(anyhow!(
                "Mount point {} must refer to an absolute path ({})",
                mount_point,
                folder
            ));
        }

        if !path.is_dir() {
            return Err(anyhow!(
                "Mount point {} must refer to a directory ({})",
                mount_point,
                folder
            ));
        }

        let mut writer = self.inner.mount_points.write().unwrap();
        {
            let mount_points = &*writer;
            if mount_points.iter().any(|x| x.name == mount_point) {
                return Err(anyhow!(
                    "Mount point {} pointing to directory ({}) already exists",
                    mount_point,
                    folder
                ));
            }
        }

        writer.push(MountPoint {
            name: mount_point.to_owned(),
            path,
        });
        Ok(())
    }

    pub fn translate_path(&self, path: &str) -> Result<BasePathBuf> {
        let protocol = "crate://";
        if !path.starts_with(protocol) {
            return Err(anyhow!("Invalid path"));
        }
        let path = &path[protocol.len()..];
        let path_parts: Vec<&str> = path.split('/').collect();
        if path_parts.is_empty() {
            return Err(anyhow!("Invalid path"));
        }
        let reader = self.inner.mount_points.read().unwrap();
        let mount_points = &*reader;
        let mut base_path = if let Some(mt) = mount_points.iter().find(|x| x.name == path_parts[0])
        {
            mt.path.clone()
        } else {
            self.inner.root_path.clone()
        };

        path_parts.iter().skip(1).for_each(|f| base_path.push(f));

        Ok(base_path)
    }

    pub fn get_file_content(&self, path: &str) -> Result<String> {
        let abs_path = self.translate_path(path).unwrap();

        dbg!(&abs_path);
        if !abs_path.is_absolute() {
            return Err(anyhow!(
                "Should be an absolute path {}",
                abs_path.as_os_str().to_str().unwrap()
            ));
        }
        if !abs_path.exists() {
            return Err(anyhow!(
                "Path {} does not exists",
                abs_path.as_os_str().to_str().unwrap()
            ));
        }
        let mut shader_code = String::new();
        std::io::Read::read_to_string(&mut std::fs::File::open(&abs_path)?, &mut shader_code)?;
        Ok(shader_code)
    }
}

pub struct FileServerIncludeHandler(pub FileSystem); // stack

impl DxcIncludeHandler for FileServerIncludeHandler {
    fn load_source(&self, filename: String) -> Option<String> {
        // Absolute file
        let inc_path = if let Some(pos) = filename.find("crate://") {
            &filename[pos..]
        } else {
            todo!()
        };

        let result = self.0.get_file_content(inc_path);
        match result {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    }
}
