use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use lgn_content_store::ContentStoreAddr;
use lgn_data_build::{DataBuild, DataBuildOptions};
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::Project;
use lgn_data_runtime::ResourceType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub code_paths: Vec<PathBuf>,
    pub project: PathBuf,
    pub buildindex: PathBuf,
    pub type_map: BTreeMap<ResourceType, String>,
}

impl Config {
    pub fn write(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let file = std::fs::File::create(path.as_ref())?;
        serde_json::to_writer_pretty(file, self).map_err(|_e| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "failed to write config")
        })?;
        Ok(())
    }

    pub fn read(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let config = serde_json::from_reader(file).map_err(|_e| {
            std::io::Error::new(std::io::ErrorKind::Other, "failed to read config")
        })?;
        Ok(config)
    }

    pub async fn open(&self) -> Result<(DataBuild, Project), String> {
        let project = Project::open(self.project.clone())
            .await
            .map_err(|e| e.to_string())?;
        let buildindex = self.buildindex.clone();
        let build = DataBuildOptions::new(buildindex, CompilerRegistryOptions::default())
            .content_store(&ContentStoreAddr::from("."))
            .open()
            .await
            .map_err(|e| e.to_string())?;

        Ok((build, project))
    }

    fn target_dir() -> PathBuf {
        std::env::current_exe().ok().map_or_else(
            || panic!("cannot find test directory"),
            |mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            },
        )
    }

    pub fn workspace_dir() -> PathBuf {
        Self::target_dir()
            .as_path()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned()
    }

    pub fn default_path() -> PathBuf {
        Self::target_dir().as_path().join("scrape-config.json")
    }
}
