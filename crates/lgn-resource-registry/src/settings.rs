use std::path::{Path, PathBuf};

use lgn_content_store::ContentStoreAddr;

#[derive(Debug, Clone)]
pub enum CompilationMode {
    InProcess,
    External,
    Remote { url: String },
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
    pub(crate) source_control_path: String,
    pub(crate) build_output_db_addr: String,
    pub(crate) content_store_addr: ContentStoreAddr,
    pub(crate) compilation_mode: CompilationMode,
}

impl ResourceRegistrySettings {
    pub fn new(
        root_folder: impl AsRef<Path>,
        source_control_path: String,
        build_output_db_addr: String,
        content_store_addr: ContentStoreAddr,
        compilation_mode: CompilationMode,
    ) -> Self {
        Self {
            root_folder: root_folder.as_ref().to_owned(),
            source_control_path,
            build_output_db_addr,
            content_store_addr,
            compilation_mode,
        }
    }

    pub fn root_folder(&self) -> &Path {
        self.root_folder.as_path()
    }
}
