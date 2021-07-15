use crate::*;
use futures::executor::block_on;
use sqlx::Connection;
use std::fs;
use std::path::{Path, PathBuf};

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

impl BlobStorageSpec{
    pub fn to_str(&self) -> &str{
        match self {
            BlobStorageSpec::LocalDirectory(dir) => {
                dir.to_str().unwrap()
            }
            BlobStorageSpec::S3Uri(uri) => {
                uri
            }
        }
    }
}

pub struct RepositoryConnection {
    blob_store: BlobStorageSpec,
    sql_connection: sqlx::AnyConnection,
}

#[derive(Debug)]
pub struct RepositoryAddr {
    pub repo_uri: String,
    pub blob_store: BlobStorageSpec,
}

impl RepositoryConnection {
    pub fn new(addr: &RepositoryAddr) -> Result<Self, String> {
        match block_on(sqlx::AnyConnection::connect(&addr.repo_uri)) {
            Err(e) => Err(format!("Error opening database {}: {}", addr.repo_uri, e)),
            Ok(c) => Ok(Self {
                blob_store: addr.blob_store.clone(),
                sql_connection: c,
            }),
        }
    }

    pub fn sql(&mut self) -> &mut sqlx::AnyConnection {
        &mut self.sql_connection
    }

    pub fn read_blob(&self, hash: &str) -> Result<String, String> {
        match &self.blob_store {
            BlobStorageSpec::LocalDirectory(dir) => {
                let blob_path = dir.join(hash);
                lz4_read(&blob_path)
            }
            BlobStorageSpec::S3Uri(_) => Err(String::from("read_blob for s3 not implemented")),
        }
    }

    pub fn download_blob(&self, local_path: &Path, hash: &str) -> Result<(), String> {
        match &self.blob_store {
            BlobStorageSpec::LocalDirectory(dir) => {
                assert!(!hash.is_empty());
                let blob_path = dir.join(hash);
                lz4_decompress(&blob_path, local_path)
            }
            BlobStorageSpec::S3Uri(_) => Err(String::from("download_blob for s3 not implemented")),
        }
    }

    pub fn write_blob(&self, hash: &str, contents: &[u8]) -> Result<(), String> {
        match &self.blob_store {
            BlobStorageSpec::LocalDirectory(dir) => {
                let path = dir.join(hash);
                write_blob_to_disk(&path, contents)
            }
            BlobStorageSpec::S3Uri(_) => Err(String::from("write_blob for s3 not implemented")),
        }
    }
}

pub fn connect_to_server(workspace: &Workspace) -> Result<RepositoryConnection, String> {
    RepositoryConnection::new(&RepositoryAddr {
        repo_uri: workspace.repo_uri.clone(),
        blob_store: BlobStorageSpec::LocalDirectory(PathBuf::from(&workspace.blob_dir)),
    })
}

fn write_blob_to_disk(file_path: &Path, contents: &[u8]) -> Result<(), String> {
    if fs::metadata(file_path).is_ok() {
        //blob already exists
        return Ok(());
    }
    lz4_compress_to_file(file_path, contents)
}
