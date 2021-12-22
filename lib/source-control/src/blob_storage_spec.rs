use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use url::Url;

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

    pub fn from_json(contents: &str) -> Result<Self> {
        serde_json::from_str(contents).context("parsing blob storage spec")
    }

    pub fn from_uri(uri: &str) -> Result<Self> {
        let parsed = Url::parse(uri).context("parsing blob storage spec")?;

        let mut bogus_path = String::from(parsed.path());
        let path = bogus_path.split_off(1); //remove leading /

        match parsed.scheme() {
            "file" => Ok(Self::LocalDirectory(PathBuf::from(path))),
            "s3" => Ok(Self::S3Uri(String::from(uri))),
            unknown => Err(anyhow::format_err!(
                "unknown blob storage scheme {}",
                unknown
            )),
        }
    }
}
