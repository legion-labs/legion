use crate::*;
use futures::executor::block_on;
use sqlx::Connection;
use std::fs;
use std::path::Path;

pub struct RepositoryConnection {
    blob_store: BlobStorageSpec,
    sql_connection: sqlx::AnyConnection,
}

pub fn connect(database_uri: &str) -> Result<sqlx::AnyConnection, String> {
    match block_on(sqlx::AnyConnection::connect(database_uri)) {
        Ok(connection) => Ok(connection),
        Err(e) => Err(format!("Error connecting to {}: {}", database_uri, e)),
    }
}

impl RepositoryConnection {
    pub fn new(repo_uri: &str) -> Result<Self, String> {
        let mut c = connect(repo_uri)?;
        Ok(Self {
            blob_store: read_blob_storage_spec(&mut c)?,
            sql_connection: c,
        })
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
    RepositoryConnection::new(&workspace.repo_uri)
}

fn write_blob_to_disk(file_path: &Path, contents: &[u8]) -> Result<(), String> {
    if fs::metadata(file_path).is_ok() {
        //blob already exists
        return Ok(());
    }
    lz4_compress_to_file(file_path, contents)
}
