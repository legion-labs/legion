use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};
use hassle_rs::DxcIncludeHandler;
use normpath::{BasePath, BasePathBuf, PathExt};
use relative_path::RelativePath;

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
            if mount_points
                .iter()
                .find(|x| x.name == mount_point)
                .is_some()
            {
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

    // pub fn get_file_content_from_rel_path(
    //     &self,
    //     path: &RelativePath,
    // ) -> Result<(BasePathBuf, String)> {
    //     let abs_path = self.get_absolute_path(path)?;
    //     self.get_file_content_from_abs_path(&abs_path)
    // }

    pub fn translate_path(&self, path: &str) -> Result<BasePathBuf> {
        let protocol = "crate://";
        if !path.starts_with(protocol) {
            return Err(anyhow!("Invalid path"));
        }
        let path = &path[protocol.len()..];
        let path_parts: Vec<&str> = path.split("/").collect();
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

    fn get_absolute_path(&self, rel_path: &RelativePath) -> Result<BasePathBuf> {
        let mut abs_path = PathBuf::new();
        let reader = self.inner.mount_points.read().unwrap();
        for src_folder in &*reader {
            // let candidate_path = rel_path.to_logical_path(&src_folder);
            // if candidate_path.exists() && !abs_path.exists() {
            //     abs_path = candidate_path;
            // } else if candidate_path.exists() && abs_path.exists() {
            //     return Err(anyhow!("Multiple occurences of the file {}", rel_path));
            // }
        }
        if !abs_path.exists() {
            return Err(anyhow!(
                "File {} does not exist in the specified folders",
                rel_path
            ));
        }
        let mut result = BasePathBuf::new(&abs_path).map_err(|e| anyhow!(e))?;
        result.canonicalize().map_err(|e| anyhow!(e))
    }

    fn to_abs_path(path: &str) -> Result<BasePathBuf> {
        let outdir_path = Path::new(path).normalize().unwrap();
        if outdir_path.is_relative() {
            let cur_dir = std::env::current_dir()?;
            BasePathBuf::new(RelativePath::new(path).to_logical_path(cur_dir))
                .map_err(|e| anyhow!(e))
        } else {
            Ok(outdir_path)
        }
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

        // let file_path = BasePath::new(Path::new(&filename)).unwrap();
        // let result = if file_path.is_absolute() {
        //     self.0.get_file_content(&file_path)
        // } else {
        //     self.0.get_file_content_from_rel_path(
        //         RelativePath::from_path(file_path.as_path()).unwrap(),
        //     )
        // };
        // match result {
        //     Ok(r) => Some(r.1),
        //     Err(_) => None,
        // }
    }
}
