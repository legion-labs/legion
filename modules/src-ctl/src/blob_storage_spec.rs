use std::path::PathBuf;

#[derive(Debug, Clone)]
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
}
