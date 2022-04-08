use std::path::{Path, PathBuf};

use lgn_source_control::RepositoryName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompilationMode {
    InProcess,
    External,
    Remote { url: String },
}

impl Default for CompilationMode {
    fn default() -> Self {
        Self::InProcess
    }
}

impl std::fmt::Display for CompilationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationMode::InProcess => write!(f, "in-process"),
            CompilationMode::External => write!(f, "external"),
            CompilationMode::Remote { url } => write!(f, "remote:{}", url),
        }
    }
}

impl std::str::FromStr for CompilationMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in-process" => Ok(Self::InProcess),
            "external" => Ok(Self::External),
            _ => {
                if let Some(url) = s.strip_prefix("remote:") {
                    Ok(Self::Remote {
                        url: url.to_string(),
                    })
                } else {
                    Err("Use: 'in-process', 'external' or 'remote:<url>'".to_string())
                }
            }
        }
    }
}

pub struct ResourceRegistrySettings {
    pub(crate) root_folder: PathBuf,
    pub(crate) source_control_repository_index: Box<dyn lgn_source_control::RepositoryIndex>,
    pub(crate) source_control_repository_name: RepositoryName,
    pub(crate) build_output_db_addr: String,
    pub(crate) compilation_mode: CompilationMode,
}

impl ResourceRegistrySettings {
    pub fn new(
        root_folder: impl AsRef<Path>,
        source_control_repository_index: Box<dyn lgn_source_control::RepositoryIndex>,
        source_control_repository_name: RepositoryName,
        build_output_db_addr: String,
        compilation_mode: CompilationMode,
    ) -> Self {
        Self {
            root_folder: root_folder.as_ref().to_owned(),
            source_control_repository_index,
            source_control_repository_name,
            build_output_db_addr,
            compilation_mode,
        }
    }

    pub fn root_folder(&self) -> &Path {
        self.root_folder.as_path()
    }
}
