use crate::{sql::*, sql_repository_query::*, *};
use std::{path::PathBuf, sync::Arc};

pub struct RepositoryConnection {
    blob_storage_spec: BlobStorageSpec,
    compressed_blob_cache: PathBuf,
    repo_query: Box<dyn RepositoryQuery + Send>,
}

impl RepositoryConnection {
    pub async fn new_sql_connection(pool: Arc<SqlConnectionPool>, compressed_blob_cache: PathBuf) -> Result<Self, String> {
        let repo_query = Box::new(SqlRepositoryQuery::new(pool).await?);
        let blob_storage_spec = repo_query.read_blob_storage_spec().await?;
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
    let pool = Arc::new(SqlConnectionPool::new(&workspace.repo_uri).await?);
    RepositoryConnection::new_sql_connection(pool, blob_cache_dir).await
}
