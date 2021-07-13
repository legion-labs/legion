use crate::*;
use futures::executor::block_on;
use sqlx::Connection;
use std::path::{Path, PathBuf};

pub struct RepositoryConnection {
    blob_directory: PathBuf, //to store blobs, will be replaced by a generic blob storage interface
    sql_connection: sqlx::AnyConnection,
}

#[derive(Debug)]
pub struct RepositoryAddr {
    pub repo_uri: String,
    pub blob_dir: PathBuf,
}

impl RepositoryConnection {
    pub fn new(addr: &RepositoryAddr) -> Result<Self, String> {
        match block_on(sqlx::AnyConnection::connect(&addr.repo_uri)) {
            Err(e) => Err(format!("Error opening database {}: {}", addr.repo_uri, e)),
            Ok(c) => Ok(Self {
                blob_directory: addr.blob_dir.clone(),
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
        blob_dir: PathBuf::from(&workspace.blob_dir),
    })
}
