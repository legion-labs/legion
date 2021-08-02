use crate::{sql::*, sql_repository_query::*, *};
use std::path::PathBuf;

pub struct RepositoryConnection {
    blob_storage_spec: BlobStorageSpec,
    compressed_blob_cache: PathBuf,
    repo_query: Box<dyn RepositoryQuery + Send>,
}

impl RepositoryConnection {
    pub async fn new(repo_uri: &str, compressed_blob_cache: PathBuf) -> Result<Self, String> {
        let repo_query = Box::new(SqlRepositoryQuery::new(repo_uri)?);
        let mut sql_connection = connect(repo_uri)?; //todo: remove
        let blob_storage_spec = read_blob_storage_spec(&mut sql_connection)?;

        Ok(Self {
            blob_storage_spec,
            compressed_blob_cache,
            repo_query,
        })
    }

    pub fn query(&self) -> &dyn RepositoryQuery {
        &*self.repo_query
    }

    pub async fn blob_storage(&self) -> Result<Box<dyn BlobStorage>, String> {
        match &self.blob_storage_spec {
            BlobStorageSpec::LocalDirectory(blob_directory) => Ok(Box::new(DiskBlobStorage {
                blob_directory: blob_directory.clone(),
            })),
            BlobStorageSpec::S3Uri(s3uri) => Ok(Box::new(
                S3BlobStorage::new(s3uri, self.compressed_blob_cache.clone()).await?,
            )),
        }
    }
}

pub async fn connect_to_server(workspace: &Workspace) -> Result<RepositoryConnection, String> {
    let blob_cache_dir = std::path::Path::new(&workspace.root).join(".lsc/blob_cache");
    RepositoryConnection::new(&workspace.repo_uri, blob_cache_dir).await
}
