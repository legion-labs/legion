use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use lgn_content_store2::ContentProvider;
use lgn_data_build::{DataBuild, DataBuildOptions};
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::Project;
use lgn_data_runtime::ResourceType;
use lgn_source_control::LocalRepositoryIndex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub code_paths: Vec<PathBuf>,
    pub project: PathBuf,
    pub output_db_addr: String,
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

    pub async fn open(
        &self,
        source_control_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>>,
        data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>>,
    ) -> Result<(DataBuild, Project), String> {
        let repository_index = LocalRepositoryIndex::new(self.project.join("remote"))
            .await
            .map_err(|e| e.to_string())?;
        let project = Project::open(
            &self.project,
            repository_index,
            Arc::clone(&source_control_content_provider),
        )
        .await
        .map_err(|e| e.to_string())?;

        let build = DataBuildOptions::new(
            DataBuildOptions::output_db_path(
                &self.output_db_addr,
                Self::workspace_dir(),
                DataBuild::version(),
            ),
            Arc::clone(&data_content_provider),
            CompilerRegistryOptions::default(),
        )
        .open(&project)
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
