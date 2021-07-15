use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlobStorageSpec {
    LocalDirectory(PathBuf),
    S3Uri(String),
}

impl std::fmt::Display for BlobStorageSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlobStorageSpec::LocalDirectory(dir) => {
                write!(f, "{}", dir.display())
            }
            BlobStorageSpec::S3Uri(uri) => {
                write!(f, "{}", uri)
            }
        }
    }
}

impl BlobStorageSpec {
    pub fn to_str(&self) -> &str {
        match self {
            BlobStorageSpec::LocalDirectory(dir) => dir.to_str().unwrap(),
            BlobStorageSpec::S3Uri(uri) => uri,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn from_json(contents: &str) -> Result<Self, String> {
        let parsed: serde_json::Result<Self> = serde_json::from_str(contents);
        match parsed {
            Ok(spec) => Ok(spec),
            Err(e) => Err(format!("Error parsing blob storage spec: {}", e)),
        }
    }
}
