use crate::*;
use futures::executor::block_on;
use http::Uri;
use sqlx::Connection;
use std::path::{Path, PathBuf};

pub struct RepositoryConnection {
    blob_directory: PathBuf, //to store blobs, will be replaced by a generic blob storage interface
    sql_connection: sqlx::AnyConnection,
}

#[derive(Debug)]
pub struct RepositoryAddr {
    pub repo_uri: String,
    pub blob_uri: String,
}

impl RepositoryConnection {
    pub fn new(addr: &RepositoryAddr) -> Result<Self, String> {
        let blob_store_uri = addr.blob_uri.parse::<Uri>().unwrap();
        assert_eq!(blob_store_uri.scheme_str(), Some("file"));
        match block_on(sqlx::AnyConnection::connect(&addr.repo_uri)) {
            Err(e) => Err(format!("Error opening database {}: {}", addr.repo_uri, e)),
            Ok(c) => Ok(Self {
                blob_directory: PathBuf::from(blob_store_uri.path()),
                sql_connection: c,
            }),
        }
    }

    pub fn sql(&mut self) -> &mut sqlx::AnyConnection {
        &mut self.sql_connection
    }

    pub fn blob_directory(&self) -> &Path {
        &self.blob_directory
    }
}

pub fn connect_to_server(workspace: &Workspace) -> Result<RepositoryConnection, String> {
    RepositoryConnection::new(&RepositoryAddr {
        repo_uri: workspace.repo_uri.clone(),
        blob_uri: workspace.blob_store_uri.clone(),
    })
}
